from multiprocessing import Pool
import os
import polars as pl
import hashlib

from constants import N_PROC, LISTS_TO_PARSE, ANONYMIZE_COLUMNS


# TODO: move to config location
INPUT_DIR_PATH = os.environ["INPUT_DIR"]
OUTPUT_DIR_PATH = os.environ["OUTPUT_DIR"]


def parse_mail_at(mailing_list):
    """
    Parses the emails from a single specified list,
    to be found in INPUT_DIR_PATH/mailing_list
    """

    input_path = INPUT_DIR_PATH + "/" + mailing_list
    outout_path = OUTPUT_DIR_PATH + "/" + mailing_list

    try:
        df = pl.read_parquet(input_path)
        if len(df) == 0:
            return
        for col in ANONYMIZE_COLUMNS:
            print(f"Running {col}")
            df = df.with_columns(
                pl.col(col)
                .map_elements(lambda x: anonymizer(x), return_dtype=pl.self_dtype())
                .alias(col),
            )

        print(f"Writing {outout_path}")

        os.makedirs(outout_path, exist_ok=True)
        df.write_parquet(outout_path + "/data.parquet")
    except Exception as e:
        print(e)


def generate_sha1_hash(input_string):
    encoded_string = input_string.encode("utf-8")
    # Create an SHA-1 hash object
    sha1_hash_object = hashlib.sha1()
    sha1_hash_object.update(encoded_string)
    # Get the hexadecimal representation of the digest
    hex_digest = sha1_hash_object.hexdigest()
    return hex_digest


def anonymizer(col):
    # print(col)
    if isinstance(col, str):
        return generate_sha1_hash(col)
    if hasattr(col, "__iter__"):
        return [generate_sha1_hash(val) for val in col]
    else:
        raise Exception(f"Unmapped type for {type(col)}")


def main():
    p = Pool(N_PROC)

    if len(LISTS_TO_PARSE) > 0:
        p.map(parse_mail_at, LISTS_TO_PARSE)
    else:
        p.map(parse_mail_at, os.listdir(INPUT_DIR_PATH))


if __name__ == "__main__":
    main()
