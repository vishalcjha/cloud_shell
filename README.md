# cloud_shell
A bundle of helpful command line utilities

## klog
Tail log in multiple pods based on pod name prefix and optional created time.

```
Usage: klog [OPTIONS] <pod-label>...

Arguments:
  <pod-label>...  Pod name prefix(s)

Options:
  -a, --catime <catime>  [+]n[h|m|s]
  -b, --cbtime <cbtime>  [+]n[h|m|s]
  -h, --help             Print help (see more with '--help')