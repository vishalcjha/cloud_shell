use clap::{Arg, Command};
const POD_NAME_MATCHER_FLAG: &'static str = "pod-name-matcher";
fn main() {
    let matches = Command::new("kgrep")
        .about("grep in multiple pods")
        .author("Vishal Kumar, vishalcjha@gmail.com")
        .arg(
            Arg::new(POD_NAME_MATCHER_FLAG)
                .value_name(POD_NAME_MATCHER_FLAG)
                .help("Pod name matcher(s)")
                .required(true)
                .num_args(1..),
        )
        .get_matches();
    let pod_regx: Vec<String> = matches
        .get_many(POD_NAME_MATCHER_FLAG)
        .expect("must provide pod to match against")
        .cloned()
        .collect();

    println!("{:?}", pod_regx);
}
