import subprocess
import time
from dataclasses import dataclass

import schedule

SERVER_PROCESS = None
LOGGER_PROCESS = None

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


def update_server(server_path: str, print: bool = False):
    command = f"steamcmd +force_install_dir {server_path} +login anonymous +app_update 232130 validate +exit"
    try:
        print("Starting the update process...")
        update_process = subprocess.Popen(command, cwd=server_path, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, bufsize=1, universal_newlines=True)
        if print:
            for line in update_process.stdout:
                print(line)
        update_process.wait()
        print("Update process finished.")
    except Exception as e:
        print(e)
    

def start_server(args: Args) -> subprocess.Popen:
    command = f"{args.server_path}/Binaries/Win64/KFGameSteamServer.bin.x86_64 kf-bioticslab"
    try:
        server_process = subprocess.Popen(command, cwd=args.server_path, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, bufsize=1, universal_newlines=True)
        return server_process
    except Exception as e:
        print(e)


def start_logger(args: Args) -> subprocess.Popen:
    command = f"RUST_LOG=info cargo run -r -- {args.web_admin_url} {args.web_admin_username} {args.web_admin_password} {args.database_url} {args.database_name} {args.database_username} {args.database_password}"
    try:
        logger_process = subprocess.Popen(command, cwd=args.logger_path, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, bufsize=1, universal_newlines=True)
        return logger_process
    except Exception as e:
        print(e)


def stop_all():
    global SERVER_PROCESS, LOGGER_PROCESS
    if SERVER_PROCESS is not None and LOGGER_PROCESS is not None:
        SERVER_PROCESS.terminate()
        SERVER_PROCESS.wait()
        LOGGER_PROCESS.terminate()
        LOGGER_PROCESS.wait()


def init_all(args: Args) -> tuple[subprocess.Popen, subprocess.Popen]:
    stop_all()
    update_server(args.server_path)
    print("Starting server...")
    server_process = start_server(args)
    time.sleep(20)
    print("Starting logger...")
    logger_process = start_logger(args)
    print("All processes started.")
    return server_process, logger_process


def main():
    global SERVER_PROCESS, LOGGER_PROCESS
    args = Args(
        
    )

    schedule.every().day.at("05:00").do(init_all, args=args)

    SERVER_PROCESS, LOGGER_PROCESS = init_all(args)

    while True:
        schedule.run_pending()
        time.sleep(10)

