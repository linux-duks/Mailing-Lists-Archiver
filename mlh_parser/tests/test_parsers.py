import pytest
import os
import io
from pathlib import Path

from mlh_parser.parser import parse_and_process_email


# helper functions
def list_files_with_extension(directory_path, extension):
    if not extension.startswith("."):
        extension = "." + extension  # Ensure the extension starts with a dot

    relpath = Path(__file__).parent.resolve()
    directory_path = relpath.joinpath(directory_path)
    files_with_extension = []
    for filename in os.listdir(directory_path):
        full_filename = os.path.join(directory_path, filename)
        if filename.endswith(extension) and os.path.isfile(full_filename):
            files_with_extension.append(full_filename)
    return files_with_extension


def map_to_files(email_file_name):
    return (
        email_file_name,
        email_file_name.rstrip(".eml") + ".code.pytest",
        email_file_name.rstrip(".eml") + ".trailers.pytest",
    )


directory = "./real_cases/"
email_files = list_files_with_extension(directory, ".eml")

real_mail_files = [map_to_files(email_f) for email_f in email_files]


@pytest.mark.parametrize("email_file, code_file, trailers_file", real_mail_files)
def test_real_mails(email_file, code_file, trailers_file) -> None:
    mail_text = io.open(email_file, mode="r", encoding="utf-8").read()
    code = eval(io.open(code_file, mode="r", encoding="utf-8").read())
    trailers = eval(io.open(trailers_file, mode="r", encoding="utf-8").read())

    output = parse_and_process_email(mail_text)

    assert output["trailers"] == trailers, f"trailers should match for {email_file}"
    assert output["code"] == code, f"code should match for {email_file}"
