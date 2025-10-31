from mlh_graph import (
    mlhid_from_person_identification,
    mlhid_from_origin_url,
    mlhid_from_content_str,
)

import hashlib


def generate_sha1_hash(input_string):
    encoded_string = input_string.encode("utf-8")
    # Create an SHA-1 hash object
    sha1_hash_object = hashlib.sha1()
    sha1_hash_object.update(encoded_string)
    # Get the hexadecimal representation of the digest
    hex_digest = sha1_hash_object.hexdigest()
    return hex_digest


def test_from_email() -> None:
    input_info = "Example Developer <example@linux.org>"

    mlhid_email = mlhid_from_person_identification(input_info)
    expected_prefix = "mlh:1:prs:"

    assert mlhid_email.startswith(expected_prefix)
    assert mlhid_email == expected_prefix + generate_sha1_hash(input_info)


def test_from_url() -> None:
    mlhid_url = mlhid_from_origin_url("linux,org")
    assert mlhid_url.startswith("mlh:1:ori:")


def test_from_body() -> None:
    mlhid_body = mlhid_from_content_str("emm", "test body \n body")
    assert mlhid_body.startswith("mlh:1:emm:")


def test_doc() -> None:
    import mlh_graph

    assert mlh_graph.__doc__ == "Mailing-Lists Heritage Graph supporting library"
