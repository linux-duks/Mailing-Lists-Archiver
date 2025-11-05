import nox


@nox.session
def python(session):
    session.install(".[dev]")
    session.run("pytest")
