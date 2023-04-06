.Tar.Gz GREP
## Why I made this tool
```terminal
$ time tar xf test_data.tar -O | rg hogeP # fast but no filename
gRRay4bho5P4hZZWvBDCX50cX2fJAyLNhogePvGaFWwaPFdmi3Y8zvJai2OLpQ13+tZB2zm8KbAI

real	0m1.392s
user	0m0.828s
sys	0m1.513s
$ tar xf test_data.tar '--to-command=grep --label=$TAR_FILENAME -H hogeP; true' # extremely slow
test_data/863227.txt:gRRay4bho5P4hZZWvBDCX50cX2fJAyLNhogePvGaFWwaPFdmi3Y8zvJai2OLpQ13+tZB2zm8KbAI

real	24m21.333s
user	20m9.365s
sys	4m32.069s
$ tzgrep hogeP test_data.tar # very fast with filename
test_data/863227.txt:gRRay4bho5P4hZZWvBDCX50cX2fJAyLNhogePvGaFWwaPFdmi3Y8zvJai2OLpQ13+tZB2zm8KbAI

real	0m0.644s
user	0m0.475s
sys	0m0.168s
```

## Usage

```terminal
$ tzgrep --help
grep tar.gz

Usage: tzgrep [OPTIONS] <PATTERN> [FILE]

Arguments:
  <PATTERN>
          search pattern [regular expression](https://crates.io/crates/regex)

  [FILE]
          search target. If not presented read from stdin.
          
          .tar, .tar.gz, .tar.bz2, .tar.xz, .tar.zst are supported

Options:
  -n, --line-number
          print line number with output lines

  -F, --fixed-string
          Asuume search pattern to be fixed string

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

