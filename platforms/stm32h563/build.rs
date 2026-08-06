use std::{collections::BTreeSet, env, fs, path::PathBuf};

#[derive(Clone, Debug)]
struct Route {
    package: String,
    pad: String,
    peripheral: String,
    signal: String,
    index: u8,
    function: u8,
    ioset: u8,
}

fn main() {
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=../../resources/devices/stmicroelectronics/stm32h563zi-lqfp144-routes.csv");

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    fs::copy("memory.x", out.join("memory.x")).expect("copy STM32H563 linker script");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=-Tmemory.x");

    let package = selected_package();
    let csv = fs::read_to_string("../../resources/devices/stmicroelectronics/stm32h563zi-lqfp144-routes.csv")
        .expect("read STMicroelectronics route data");
    let routes = parse_routes(&csv);
    let generated = generate_routes(package, &routes);
    fs::write(out.join("routes_generated.rs"), generated).expect("write generated package routes");
}

fn selected_package() -> &'static str {
    let lqfp144 = env::var_os("CARGO_FEATURE_PACKAGE_LQFP144").is_some();
    match (lqfp144) {
        (true) => "LQFP144",
        (false) => panic!(
            "select exactly one STM32H563 package feature: package-lqfp144"
        ),
        (true) => panic!("STM32H563 package features are mutually exclusive"),
    }
}

fn parse_routes(csv: &str) -> Vec<Route> {
    csv.lines()
        .filter(|line| {
            !line.is_empty() && !line.starts_with('#') && !line.starts_with("\"package\"")
        })
        .map(|line| {
            let fields: Vec<_> = line
                .split(',')
                .map(|field| field.trim_matches('"'))
                .collect();
            assert_eq!(fields.len(), 9, "unexpected route CSV row: {line}");
            Route {
                package: fields[0].to_owned(),
                pad: fields[3].to_owned(),
                peripheral: fields[4].to_owned(),
                signal: fields[5].to_owned(),
                index: fields[6].parse().unwrap_or(0),
                function: function_number(fields[7]),
                ioset: fields[8].parse().unwrap_or(0),
            }
        })
        .collect()
}

fn function_number(function: &str) -> u8 {
    let bytes = function.as_bytes();
    if bytes.len() == 1 && (b'A'..=b'N').contains(&bytes[0]) {
        bytes[0] - b'A'
    } else {
        0
    }
}

fn generate_routes(package: &str, routes: &[Route]) -> String {
    let mut output = format!(
        "// Generated from STMicroelectronics STM32H563 DFP. Do not edit.\n\
         pub(crate) const PACKAGE: &str = \"{package}\";\n"
    );
    output.push_str(&generate_fn(
        "adc0",
        "AnalogRoute",
        routes,
        package,
        "ADC0",
        "AIN",
        |route| {
            format!(
                "AnalogRoute {{ channel: {}, function: {} }}",
                route.index, route.function
            )
        },
    ));
    output.push_str(&generate_fn(
        "dac1",
        "AnalogRoute",
        routes,
        package,
        "DAC",
        "VOUT",
        |route| {
            format!(
                "AnalogRoute {{ channel: {}, function: {} }}",
                route.index, route.function
            )
        },
    ));
    output.push_str(&generate_fn(
        "tc4",
        "TimerRoute",
        routes,
        package,
        "TC4",
        "WO",
        |route| {
            format!(
                "TimerRoute {{ waveform_output: {}, function: {} }}",
                route.index, route.function
            )
        },
    ));
    output.push_str(&generate_fn(
        "eic",
        "EicRoute",
        routes,
        package,
        "EIC",
        "EXTINT",
        |route| {
            format!(
                "EicRoute {{ extint: {}, function: {} }}",
                route.index, route.function
            )
        },
    ));
    for peripheral in ["GPIO"] {
        output.push_str(&generate_fn(
            &peripheral.to_ascii_lowercase(),
            "GPIORoute",
            routes,
            package,
            peripheral,
            "PAD",
            |route| {
                format!(
                    "GPIORoute {{ pad: {}, function: {}, ioset: {} }}",
                    route.index, route.function, route.ioset
                )
            },
        ));
    }
    output
}

fn generate_fn(
    name: &str,
    result: &str,
    routes: &[Route],
    package: &str,
    peripheral: &str,
    signal: &str,
    value: impl Fn(&Route) -> String,
) -> String {
    let mut arms = BTreeSet::new();
    for route in routes.iter().filter(|route| {
        route.package == package && route.peripheral == peripheral && route.signal == signal
    }) {
        let Some((port, pin)) = parse_pad(&route.pad) else {
            continue;
        };
        arms.insert(format!(
            "        (Port::{port}, {pin}) => Some({}),\n",
            value(route)
        ));
    }
    let arms: String = arms.into_iter().collect();
    format!(
        "pub(crate) const fn {name}(pin: Pin) -> Option<{result}> {{\n\
         \x20   match (pin.port(), pin.index()) {{\n{arms}\
         \x20       _ => None,\n\
         \x20   }}\n\
         }}\n"
    )
}

fn parse_pad(pad: &str) -> Option<(char, u8)> {
    let bytes = pad.as_bytes();
    if bytes.len() != 4 || bytes[0] != b'P' || !(b'A'..=b'D').contains(&bytes[1]) {
        return None;
    }
    Some((bytes[1] as char, pad[2..].parse().ok()?))
}
