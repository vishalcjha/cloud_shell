use std::fmt::Display;

use clap::{Arg, Command};

use crate::ctime::{CTime, CTimeParser};

const POD_LABEL_FLAG: &'static str = "pod-label";
const CTIME_HELP: &'static str = r#"
If no units are specified, this primary evaluates to true if the difference between the
time of creation of pod and the time klog was started, rounded up to
the next full 24-hour period, is n 24-hour periods.
NEWLINE
If units are specified, this primary evaluates to true if the difference between the time
of creation of pod and the time klog was started is exactly n units.
"#;

#[derive(Debug)]
pub struct Args {
    pub(crate) pod_name_matchers: Vec<String>,
    pub(crate) ctime: Option<CTime>,
}

impl Args {
    pub(crate) fn read_command_line() -> Args {
        let matches = Command::new("kgrep")
            .about("Tail log in multiple pods based on pod name prefix and optional created time.")
            .author("Vishal Kumar, vishalcjha@gmail.com")
            .arg(
                Arg::new(POD_LABEL_FLAG)
                    .value_name(POD_LABEL_FLAG)
                    .help("Pod name prefix(s)")
                    .required(true)
                    .num_args(1..),
            )
            .arg(
                Arg::new("catime")
                    .long("catime")
                    .short('a')
                    .required(false)
                    .conflicts_with("cbtime")
                    .help("[+]n[h|m|s]")
                    .long_help(
                        CTIME_HELP
                            .replace("\n", "")
                            .replace("\r", "")
                            .replace("NEWLINE", "\n"),
                    )
                    .value_parser(CTimeParser("a".to_string())),
            )
            .arg(
                Arg::new("cbtime")
                    .long("cbtime")
                    .short('b')
                    .required(false)
                    .conflicts_with("catime")
                    .help("[+]n[h|m|s]")
                    .long_help(
                        CTIME_HELP
                            .replace("\n", "")
                            .replace("\r", "")
                            .replace("NEWLINE", "\n"),
                    )
                    .value_parser(CTimeParser("b".to_string())),
            )
            .get_matches();

        let pod_labels: Vec<String> = matches
            .get_many(POD_LABEL_FLAG)
            .expect("must provide pod to match against")
            .cloned()
            .collect();

        let mut ctime = matches.get_one::<CTime>("catime").map(|it| it.clone());
        ctime = ctime.or_else(|| matches.get_one::<CTime>("cbtime").map(|it| it.clone()));

        Self {
            pod_name_matchers: pod_labels,
            ctime,
        }
    }
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Will monitor pod name prefixed {}",
            self.pod_name_matchers.join(","),
        ))?;

        self.ctime
            .as_ref()
            .map(|it| f.write_str(&format!(" and created {:?}", it)));
        Ok(())
    }
}
