from multiprocessing import Pool
import os

from parser import parse_mail_at
from constants import (
    N_PROC,
    LISTS_TO_PARSE,
)


# TODO: move to config location
INPUT_DIR_PATH = os.environ["INPUT_DIR"]
OUTPUT_DIR_PATH = os.environ["OUTPUT_DIR"]
PARQUET_DIR_PATH = OUTPUT_DIR_PATH + "/parsed"
PARQUET_FILE_NAME = "list_data.parquet"


def main():
    p = Pool(N_PROC)

    if len(LISTS_TO_PARSE) > 0:
        p.map(parse_mail_at, LISTS_TO_PARSE)
    else:
        p.map(parse_mail_at, os.listdir(INPUT_DIR_PATH))


if __name__ == "__main__":
    main()
