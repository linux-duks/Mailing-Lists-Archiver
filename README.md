# Mailing Lists Archiver - Track Mailing Lists over NNTP into local files

Collect and archive locally all emails from mailing lists.
This is in active development. It currently supports reading from NNTP endpoints.

# Usage

The most basic way to run this program, is to provide the NNTP Hostname and port via env variables. `NNTP_HOSTNAME="rcpassos.me" NNTP_PORT=119 cargo run`, or via arguments : `cargo run -H rcpassos.me -p 119` (note: this website is not a NNTP server).
The list of available news groups in the server will be provided for selection.

A config file can be used too (check the Example below).
By default, this program will look for the config file in the current directory.
It will look for nntp_config*.{json,yaml,toml}.

A custom config file path can be passed with the flag `-c`. Ex: `cargo run  -c other_nntp_config.yaml`

```bash
Usage: mailing-lists-archiver [OPTIONS]

Options:
  -c, --config-file <CONFIG_FILE>      [default: nntp_config*]
  -H, --hostname <HOSTNAME>            
  -p, --port <PORT>                    [default: 119]
  -o, --output-dir <OUTPUT_DIR>        [default: ./output]
  -n, --nthreads <NTHREADS>            [default: 1]
      --group-lists <GROUP_LISTS>      
      --article-range <ARTICLE_RANGE>  comma separated values, or dash separated ranges, like low-high
  -h, --help                           Print help
```

The `RUST_LOG=debug` variable can be used to increase logging details.

args: `cargo run -- -c offnntp_config.yaml -H rcpassos.me -p 119`

### Example config file

```yaml
# nntp_config.yaml
hostname: "rcpassos.me"
port: 119
nthreads: 2
output_dir: "./output"
group_lists:
  - dev.rcpassos.me.lists.gfs2
  - dev.rcpassos.me.lists.iommu
```

## Implemantation

This is the basic algorithm used by this script
![fluxogram](./docs/fluxogram.svg)


# Email Parser

Yay!

## Usage
First, build and deploy the docker container.
```bash
docker compose up
```

By default, the parser container will be up awaiting for connections. Open a bash in the parser container doing:
```bash
docker exec -it mlarchive-parser-1 /bin/bash
```

Then, run the main parser script:
```bash
python3 /parser/src/parser_main.py
```

The parsed emails will be saved in a parquet formatted archive (using hive partitioning on the name of the mailing lists) in the `parser_output/parsed/` directory.

Incorrectly parsed email will be `parser_output/<mailing_list>/errors` directory.
