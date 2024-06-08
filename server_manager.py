import datetime
import logging
import os
import subprocess
import sys
import time
from functools import cached_property

import schedule
from pydantic import BaseModel
from pydantic_settings import BaseSettings, SettingsConfigDict

log_dir = "logs"
if not os.path.exists(log_dir):
    os.makedirs(log_dir)

logging.basicConfig(
    filename="logs/server_manager.log",
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)

SERVER_PROCESS = None
LOGGER_PROCESS = None
POPEN_KWARGS = None


class PopenKwargs(BaseModel):
    shell: bool
    bufsize: int
    universal_newlines: bool
    encoding: str

    @cached_property
    def dict(self):
        return self.model_dump()


class Args(BaseSettings):
    server_path: str
    logger_path: str

    model_config = SettingsConfigDict(env_file=".env", extra="ignore")


def get_date_str() -> str:
    return datetime.date.today().isoformat().replace("-", "")


def set_firewall_game():
    ports = (7777, 27015, 8080, 20560)
    protocols = ("udp", "udp", "tcp", "udp")

    for port, protocol in zip(ports, protocols):
        command = f"sudo ufw allow {port}/{protocol}"
        command = command.split(" ")
        try:
            logging.info(f"Opening {protocol} port {port}...")
            subprocess.Popen(command, **POPEN_KWARGS.dict).wait()
            logging.info(f"Port {port} opened.")
        except Exception as e:
            logging.error(e)


def set_firewall_ttl():
    command = "sudo iptables -I INPUT -p udp --dport 7777:7778 -m ttl --ttl-gt 200 --jump DROP"
    command = command.split(" ")
    try:
        logging.info("Setting firewall TTL...")
        subprocess.Popen(command, **POPEN_KWARGS.dict).wait()
        logging.info("Firewall TTL set.")
    except Exception as e:
        logging.error(e)


def update_server(server_path: str):
    command = f"/usr/games/steamcmd +force_install_dir {server_path} +login anonymous +app_update 232130 validate +exit"
    command = command.split(" ")
    date = get_date_str()
    try:
        logging.info("Starting the update process...")
        with open(f"logs/server_update_{date}.log", "w") as update_log:
            subprocess.Popen(
                command,
                cwd=server_path,
                stderr=update_log,
                stdout=update_log,
                **POPEN_KWARGS.dict,
            ).wait()
        logging.info("Update process finished.")
    except Exception as e:
        logging.error(e)


def start_server(args: Args) -> subprocess.Popen[str]:
    command = (
        f"{args.server_path}/Binaries/Win64/KFGameSteamServer.bin.x86_64 kf-bioticslab"
    )
    command = command.split(" ")
    date = get_date_str()
    try:
        with open(f"logs/server_output_{date}.log", "w") as server_log:
            server_process = subprocess.Popen(
                command,
                cwd=args.server_path,
                stderr=server_log,
                stdout=server_log,
                **POPEN_KWARGS.dict,
            )
        return server_process
    except Exception as e:
        logging.error(e)


def start_logger(args: Args) -> subprocess.Popen[str]:
    command = "/home/jijitsu/.cargo/bin/cargo run -r"
    command = command.split(" ")
    env = os.environ
    env["RUST_LOG"] = "info"
    date = get_date_str()
    try:
        with open(f"logs/logger_output_{date}.log", "w") as logger_log:
            logger_process = subprocess.Popen(
                command,
                cwd=args.logger_path,
                env=env,
                stderr=logger_log,
                stdout=logger_log,
                **POPEN_KWARGS.dict,
            )
        return logger_process
    except Exception as e:
        logging.error(e)


def stop_all():
    global SERVER_PROCESS, LOGGER_PROCESS
    if SERVER_PROCESS and LOGGER_PROCESS:
        logging.info("Shutting down all processes...")
        SERVER_PROCESS.terminate()
        SERVER_PROCESS.wait()
        SERVER_PROCESS = None
        LOGGER_PROCESS.terminate()
        LOGGER_PROCESS.wait()
        LOGGER_PROCESS = None
        logging.info("All processes shut down.")


def init_all(args: Args):
    global SERVER_PROCESS, LOGGER_PROCESS
    try:
        stop_all()
        update_server(args.server_path)
        logging.info("Starting server...")
        server_process = start_server(args)
        time.sleep(20)
        logging.info("Starting logger...")
        logger_process = start_logger(args)
        logging.info("All processes started.")
        SERVER_PROCESS = server_process
        LOGGER_PROCESS = logger_process
    except Exception as e:
        logging.error(e)
        stop_all()
        sys.exit()


def main():
    global POPEN_KWARGS

    logging.info(f"PATH: {os.environ['PATH']}")
    logging.info(f"Current directory: {os.getcwd()}")
    logging.info(f"User ID: {os.getuid()}")

    kf2_args = Args()

    POPEN_KWARGS = PopenKwargs(
        shell=False,
        bufsize=1,
        universal_newlines=True,
        encoding="utf-8",
    )

    schedule.every().day.at("06:00").do(init_all, args=kf2_args)

    set_firewall_game()
    set_firewall_ttl()
    init_all(kf2_args)

    while True:
        schedule.run_pending()
        time.sleep(10)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        logging.error(e)
        stop_all()
        sys.exit()
