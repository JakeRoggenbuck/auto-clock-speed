import subprocess
import time
import datetime


def log_internal(msg):
    _, bat, _ = subprocess.check_output(
        "acs get power --raw",
        shell=True,
    ).decode().split()

    time = datetime.datetime.now()
    print(f"[LOG ({time}) bat={bat}]: {msg}")

    return f"{time},{bat},{msg}\n"


def log(msg):
    out = log_internal(msg)
    with open("output.log", "a") as file:
        file.write(out)


def test():
    while 1:
        log("Running...")
        time.sleep(10)

        _, bat, _ = subprocess.check_output(
            "acs get power --raw",
            shell=True,
        ).decode().split()

        if int(bat) <= 43:
            log("Test ended!")
            exit(1)


def start_test():
    log("Test Started!")

    test()


if __name__ == "__main__":
    lid, bat, ac = subprocess.check_output(
        "acs get power --raw",
        shell=True,
    ).decode().split()

    while 1:
        if int(bat) == 45 and ac == "false":
            start_test()
        else:
            print("Waiting to start test...")

        time.sleep(10)
