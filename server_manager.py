import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from functools import cached_property

import schedule

SERVER_PROCESS = None
LOGGER_PROCESS = None
POPEN_KWARGS = None


@dataclass
class PopenKwargs:
    shell: bool
    bufsize: int
    universal_newlines: bool
    encoding: str

    @cached_property
    def dict(self):
        return asdict(self)


@dataclass(slots=True)
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
    command = command.split(" ")
    try:
        print("Setting firewall TTL...")
        subprocess.Popen(command, **POPEN_KWARGS.dict).wait()
        print("Firewall TTL set.")
    except Exception as e:
        print(e)


def update_server(server_path: str, output: int = subprocess.DEVNULL):
    command = f"steamcmd +force_install_dir {server_path} +login anonymous +app_update 232130 validate +exit"
    command = command.split(" ")
    try:
        print("Starting the update process...")
        subprocess.Popen(command, cwd=server_path, stderr=output, stdout=output, **POPEN_KWARGS.dict).wait()
        print("Update process finished.")
    except Exception as e:
        print(e)


def start_server(args: Args) -> subprocess.Popen[str]:
    command = f"{args.server_path}/Binaries/Win64/KFGameSteamServer.bin.x86_64 kf-bioticslab"
    command = command.split(" ")
    try:
        server_process = subprocess.Popen(command, cwd=args.server_path, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL, **POPEN_KWARGS.dict)
        return server_process
    except Exception as e:
        print(e)


def start_logger(args: Args) -> subprocess.Popen[str]:
    command = f"cargo run -r -- {args.web_admin_url} {args.web_admin_username} {args.web_admin_password} {args.database_url} {args.database_name} {args.database_username} {args.database_password}"
    command = command.split(" ")
    env = os.environ
    env["RUST_LOG"] = "info"
    try:
        logger_process = subprocess.Popen(command, cwd=args.logger_path, env=env, **POPEN_KWARGS.dict)
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
    try:
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
    except:
        stop_all()
        sys.exit()


def main():
    global POPEN_KWARGS
    kf2_args = Args(
        
    )

    POPEN_KWARGS = PopenKwargs(
        shell=False,
        bufsize=1,
        universal_newlines=True,
        encoding="utf-8",
    )

    schedule.every().day.at("06:00").do(init_all, args=kf2_args)

    set_firewall_ttl()
    init_all(kf2_args)

    while True:
        schedule.run_pending()
        time.sleep(10)


if __name__ == "__main__":
    try:
        main()
    except:
        stop_all()
        sys.exit()
