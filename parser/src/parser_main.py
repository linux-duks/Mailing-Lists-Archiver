import io
import os
import re
import polars as pl
from datetime import datetime
from multiprocessing import Pool
from tqdm import tqdm

from parser_algorithm import parse_email_txt_to_dict
from constants import PARQUET_COLS_SCHEMA, FORCE_REPARSE, REDO_FAILED_PARSES,\
     N_PROC, SINGLE_VALUED_COLS, LISTS_TO_PARSE

INPUT_DIR_PATH = os.environ["INPUT_DIR"]
OUTPUT_DIR_PATH = os.environ["OUTPUT_DIR"]
PARQUET_DIR_PATH = OUTPUT_DIR_PATH + "/parsed"
PARQUET_FILE_NAME = "list_data.parquet"

def parse_mail_at(mailing_list):
    """
    Parses the emails from a single specified list,
    to be found in INPUT_DIR_PATH/mailing_list
    """

    list_input_path = INPUT_DIR_PATH + "/" + mailing_list
    list_output_path = OUTPUT_DIR_PATH + "/" + mailing_list
    success_output_path = PARQUET_DIR_PATH + "/list=" + mailing_list
    parquet_path = success_output_path + "/" + PARQUET_FILE_NAME
    error_output_path = list_output_path + "/errors"

    all_parsed = pl.DataFrame(schema=PARQUET_COLS_SCHEMA)

    if not os.path.isdir(list_output_path):
        print("First parse of list", mailing_list)

        if not os.path.isdir(PARQUET_DIR_PATH):
          os.mkdir(PARQUET_DIR_PATH)

        os.mkdir(list_output_path)
        os.mkdir(success_output_path)
        os.mkdir(error_output_path)
    else:
        if REDO_FAILED_PARSES:
            try:
                all_parsed = pl.read_parquet(parquet_path)
            except FileNotFoundError:
                pass      
        else:
            remove_previous_errors(error_output_path)
    all_parsed = all_parsed.with_row_index()

    if REDO_FAILED_PARSES:
        all_emails = os.listdir(error_output_path)
    else:
        all_emails = os.listdir(list_input_path)
        all_emails.remove("__last_article_number")
        if "errors.md" in all_emails:
            all_emails.remove("errors.md")

    newly_parsed = pl.DataFrame(schema=PARQUET_COLS_SCHEMA) 
    newly_parsed = newly_parsed.with_row_index()

    for email_name in tqdm(all_emails):
        email_path = list_input_path + "/" + email_name
        email_file = io.open(email_path, mode="r", encoding="utf-8")

        try:
            email_as_dict = parse_email_txt_to_dict(email_file.read())
            email_as_dict = post_process_parsed_mail(email_as_dict)
        except Exception as parsing_error:
            save_unsuccessful_parse(email_file, parsing_error, email_name, mailing_list, error_output_path)
            continue

        email_as_df = pl.DataFrame(email_as_dict,schema=PARQUET_COLS_SCHEMA)
        email_as_df = email_as_df.with_columns( # Let's keep our datetimes naive
            pl.col("date").dt.replace_time_zone(None)
        )

        email_as_df = email_as_df.with_row_index()
        newly_parsed.extend(email_as_df) # Simply adds to end of DF

        email_file.close()

        if REDO_FAILED_PARSES:
            os.remove(error_output_path + '/' + email_name)

    all_parsed.extend(newly_parsed)
    all_parsed = all_parsed.drop("index")
    all_parsed.write_parquet(parquet_path)
    print("Saved all parsed mail on list", mailing_list)
    #print(all_parsed)

def post_process_parsed_mail(email_as_dict: dict):
    """
    Post-processes dict containing email fields, parsing
    multiple valued fields and other non Str fields.
    """         

    if isinstance(email_as_dict["cc"],str):
        email_as_dict["cc"] = email_as_dict["cc"].split(',')


    #SINGLE_VALUED_COLS

    if isinstance(email_as_dict["to"],list):
        email_as_dict["cc"] = email_as_dict["to"][1:] + email_as_dict["cc"]
        email_as_dict["to"] = email_as_dict["to"][0]

    for column in SINGLE_VALUED_COLS:
        if isinstance(email_as_dict[column],list):
            email_as_dict[column] = email_as_dict[column][0]
            # This usually doesn't make sense
            # For dates, we're saving the first date parsed
 
    email_as_dict["raw_body"] = email_as_dict["body"] + email_as_dict["trailers"] + email_as_dict["code"]

    if isinstance(email_as_dict["references"],str):
        email_as_dict["references"] = email_as_dict["references"].split(' ')

    if isinstance(email_as_dict["trailers"],str):
        email_as_dict["trailers"] = email_as_dict["trailers"].split(',')

    # TODO: Anonymize everything here

    old_date_time = email_as_dict["date"]

    if '(' in old_date_time:
        old_date_time = old_date_time[:old_date_time.index('(')].strip()

    try:
        new_date_time = datetime.strptime(old_date_time, "%a, %d %b %Y %X %z")
    except Exception:
        try:
            new_date_time = datetime.strptime(old_date_time.strip(), "%a, %d %b %Y %H:%M %z")
        except Exception:
            try:
                new_date_time = datetime.strptime(old_date_time.strip(), "%d %b %Y %X %z")
            except Exception:
                new_date_time = datetime.strptime(old_date_time[:-4].strip(), "%a, %d %b %Y %X")

    email_as_dict["date"] = new_date_time

    for dict_key in email_as_dict:
        email_as_dict[dict_key] = [email_as_dict[dict_key]]

    return email_as_dict

def get_email_id(email_file) -> str:
    """
    Retrieves the email Message-ID.
    """

    for line in email_file.readlines():
        if re.match(r"^Message-ID:", line,re.IGNORECASE):
            message_id = line[len("Message-ID:"):].strip()
            email_file.seek(0,os.SEEK_SET)
            return message_id
        
    email_file.seek(0,os.SEEK_SET) # Return to the beginning of file stream
    
    raise Exception("Found email with no Message-ID field for file " + email_file.name)

def email_previously_parsed(all_parsed,email_id) -> int | None:
    """
    Checks whether the given email message id corresponds
    to a email saved in the archive. If that's the case, 
    returns the dataframe row where the email is stored.
    Otherwise, returns None.
    """
    
    filter_res = all_parsed.filter(pl.col('message-id') == email_id)

    if filter_res.shape[0] == 0:
        return None
    elif filter_res.shape[0] > 1:
        raise Exception("Message-ID conflict on parquet databse for id " + email_id)

    return filter_res[0,'index']

def save_unsuccessful_parse(email_file, parsing_error, email_name, mailing_list, error_output_path):
    """
    Saves information on unsuccessful email parse. Both original email content and
    exception information are stored in the directory at <error_output_path>, in
    a file with the same name as the original .eml file.
    """
    email_file.seek(0,os.SEEK_SET) # Return to the beginning of file stream
    
    to_save = email_file.read()
    to_save += '\n' + '='*30 + " Exception:\n"
    to_save += str(parsing_error)

    print("Error when parsing file", email_name, "of list", mailing_list)
    print(parsing_error)

    with open(error_output_path + '/' + email_name,"w",encoding="utf-8") as error_output_file:
        error_output_file.write(to_save)

    email_file.close()

def remove_previous_errors(errors_dir_path):
    """
    Removes every file from the directory at the path given.
    """

    for error_file_name in os.listdir(errors_dir_path):
        os.remove(errors_dir_path + '/' + error_file_name)

def main():

    p = Pool(N_PROC)

    if len(LISTS_TO_PARSE) > 0:
        p.map(parse_mail_at,LISTS_TO_PARSE)
    else:
        p.map(parse_mail_at,os.listdir(INPUT_DIR_PATH))

if __name__ == "__main__":
    main()
