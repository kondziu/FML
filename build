#!/bin/bash

which curl  1>/dev/null 2>/dev/null || sudo apt install curl
which cargo 1>/dev/null 2>/dev/null || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)

cargo build --release