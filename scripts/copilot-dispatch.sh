#!/usr/bin/env bash
# Nested copilot dispatch wrapper — loads auth from env file
source ~/.env_tokens
exec copilot "$@"
