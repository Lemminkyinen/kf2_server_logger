import subprocess
import time
from dataclasses import dataclass

import schedule

SERVER_PROCESS = None
LOGGER_PROCESS = None
POPEN_KWARGS = None


@dataclass
class PopenKwargs:
    shell: bool
    stdout: int
    stderr: int
    bufsize: int
    check: bool
    universal_newlines: bool
    encoding: str


@dataclass
class Args:
    server_path: str
    logger_path: str
    web_admin_url: str
    web_admin_username: str
    web_admin_password: str
    database_url: str
    database_name: str
    database_username: str
    database_password: str


def set_firewall_ttl():
    command = "sudo iptables -I INPUT -p udp --dport 7777:7778 -m ttl --ttl-gt 200 --jump DROP"
    try:
        print("Setting firewall TTL...")
        subprocess.run(command, **POPEN_KWARGS)
        print("Firewall TTL set.")
    except Exception as e:
        print(e)


def update_server(server_path: str, print_line: bool = False):
    command = f"steamcmd +force_install_dir {server_path} +login anonymous +app_update 232130 validate +exit"
    try:
        print("Starting the update process...")
        update_process = subprocess.Popen(command, cwd=server_path, **POPEN_KWARGS)
        if print_line:
            for line in update_process.stdout:
                print(line)
        update_process.wait()
        print("Update process finished.")
    except Exception as e:
        print(e)


def start_server(args: Args) -> subprocess.Popen:
    command = f"{args.server_path}/Binaries/Win64/KFGameSteamServer.bin.x86_64 kf-bioticslab"
    try:
        server_process = subprocess.Popen(command, cwd=args.server_path, **POPEN_KWARGS)
        return server_process
    except Exception as e:
        print(e)


def start_logger(args: Args) -> subprocess.Popen:
    command = f"RUST_LOG=info cargo run -r -- {args.web_admin_url} {args.web_admin_username} {args.web_admin_password} {args.database_url} {args.database_name} {args.database_username} {args.database_password}"
    try:
        logger_process = subprocess.Popen(command, cwd=args.logger_path, **POPEN_KWARGS)
        return logger_process
    except Exception as e:
        print(e)


def stop_all():
    global SERVER_PROCESS, LOGGER_PROCESS
    if SERVER_PROCESS and LOGGER_PROCESS:
        print("Shutting down all processes...")
        SERVER_PROCESS.terminate()
        SERVER_PROCESS.wait()
        SERVER_PROCESS = None
        LOGGER_PROCESS.terminate()
        LOGGER_PROCESS.wait()
        LOGGER_PROCESS = None
        print("All processes shut down.")


def init_all(args: Args):
    global SERVER_PROCESS, LOGGER_PROCESS
    stop_all()
    update_server(args.server_path)
    print("Starting server...")
    server_process = start_server(args)
    time.sleep(20)
    print("Starting logger...")
    logger_process = start_logger(args)
    print("All processes started.")
    SERVER_PROCESS = server_process
    LOGGER_PROCESS = logger_process


def main():
    global POPEN_KWARGS
    kf2_args = Args(
    )

    POPEN_KWARGS = PopenKwargs(
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        bufsize=1,
        universal_newlines=True,
        encoding="utf-8",
        check=True
    )

    schedule.every().day.at("05:00").do(init_all, args=kf2_args)

    set_firewall_ttl()
    init_all(kf2_args)

    while True:
        schedule.run_pending()
        time.sleep(10)


if __name__ == "__main__":
    main()
