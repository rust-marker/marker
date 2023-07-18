# This file is meant to be sourced by other scripts, not executed directly.
# It contains a bunch of helper functions for writing bash scripts.

set -euo pipefail

# This output stream is used by subshells to send their output to the
# global build process stdout. This is needed e.g. for writing special commands
# to the shells actual stdout instead of the command's stdout to let the github
# actions runner read them
global_stdout=3
eval "exec $global_stdout>&1"

# Retry a command a with backoff.
#
# The retry count is given by ATTEMPTS (default 5), the
# initial backoff timeout is given by TIMEOUT in seconds
# (default 1.)
#
# Successive backoffs double the timeout.
#
# Beware of set -e killing your whole script!
#
# Adapted from https://coderwall.com/p/--eiqg/exponential-backoff-in-bash
function with_backoff {
    local max_attempts=${ATTEMPTS-5}
    local timeout=${TIMEOUT-1}
    local attempt=0
    local exit_code=0

    while [[ $attempt < $max_attempts ]]
    do
        if [[ $attempt == 0 ]]; then
            start_group "${@}"
        else
            start_group "[Try $((attempt + 1))/$max_attempts] ${@}"
        fi

        # Temporarily disable the "exit script on error" behavior
        set +o errexit

        "$@"
        exit_code=$?

        # put exit on error back up
        set -o errexit

        end_group

        if [[ $exit_code == 0 ]]; then
            break
        fi

        echo "Failure! Retrying in $timeout seconds.." 1>&2
        sleep $timeout
        attempt=$(( attempt + 1 ))
        timeout=$(( timeout * 2 ))
    done

    if [[ $exit_code != 0 ]]; then
        echo "You've failed me for the last time! (exit code $exit_code) ($@)" 1>&2
    fi

    return $exit_code
}

# Returns a command with syntax highlighting
function colorize_command {
    program=$1
    shift

    local args=()
    for arg in "$@"; do
        if [[ $arg =~ ^- ]]; then
            args+=("\033[34;1m${arg}\033[0m")
        else
            args+=("\033[0;33m${arg}\033[0m")
        fi
    done

    echo "\033[1;33m${program}\033[0m ${args[*]}"
}

# Log the command and execute it
function with_log {
    command=$(colorize_command "$@")

    >&$global_stdout echo -e "\033[32;1m❱\033[0m $command"

    "$@"
}

# Log the command and execute it, but collapse the output on CI
function with_collapsed_log {
    start_group "$@"

    # Temporarily disable the "exit script on error" behavior
    set +o errexit

    "$@"
    local exit_code=$?

    # put exit on error back up
    set -o errexit

    end_group

    return $exit_code
}

function group_header {
    command=$(colorize_command "$@")

    echo -e "\033[32;1m👉 ❱❱❱❱ $command\n"
}

# Begin a collapsible group. You'll need to click on the logs to expand them on CI
#
# Beware that it's not possible to nest groups. Two consecutive start_group calls are wrong.
function start_group {
    local group=$(group_header "${@}")

    if [ "${GITHUB_ACTIONS:-false}" == "true" ]; then
        >&$global_stdout echo -e "::group::${group}"
    else
        >&$global_stdout echo -e "${group}"
    fi
}

# Finish the previously started collapsible group.
#
# Beware that it's not possible to nest groups, this closes all groups
function end_group {
    if [ "${GITHUB_ACTIONS:-false}" == "true" ]; then
        >&$global_stdout echo "::endgroup::"
    fi
}
