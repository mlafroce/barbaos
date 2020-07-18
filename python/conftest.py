import subprocess

def pytest_configure():
    subprocess.run(["cargo", "build"])
