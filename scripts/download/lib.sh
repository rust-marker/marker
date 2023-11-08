script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

# Produces Rust-style arch names, e.g. x86_64, aarch64, etc.
export arch_rust=$(uname -m)

case "$OSTYPE" in
    linux*)  export os=linux ;;
    darwin*) export os=darwin ;;
    msys)    export os=windows ;;
    *)       echo "Unknown OS: $OSTYPE" && exit 1 ;;
esac

case $os in
    linux)   export triple_rust=unknown-linux-gnu ;;
    darwin)  export triple_rust=apple-darwin ;;
    windows) export triple_rust=pc-windows-msvc ;;
esac

triple_rust=$arch_rust-$triple_rust

if [[ os == "windows" ]]; then
    export exe=.exe
else
    export exe=
fi

function download_and_decompress {
    with_backoff try_download_and_decompress "$@"
}

function try_download_and_decompress {
    local hash_algo=""
    while [[ "$#" -gt 0 ]]; do
        case $1 in
        --check-hash)
            hash_algo="$2"
            shift 2
            ;;
        *)
            break
            ;;
        esac
    done

    local url="$1"
    shift

    local archive=$(basename $url)

    curl_with_retry $url --remote-name

    # Check the hash of the downloaded file if it was requested
    if [[ "$hash_algo" != "" ]]
    then
        hash=$(curl_with_retry $url.$hash_algo)
        echo "$hash $archive" | with_log ${hash_algo}sum --check
    fi

    if [[ $url == *.tar.gz || $url == *.tgz ]]
    then
        with_log tar --extract --gzip --file $archive "$@"
    elif [[ $url == *.tar.xz ]]
    then
        with_log tar --extract --xz --file $archive "$@"
    elif [[ $url == *.gz ]]
    then
        with_log gzip --decompress --stdout $archive > $(basename $url .gz)
    else
        echo "Unknown file type: $url"
        exit 1
    fi

    rm $archive
}

# Be careful to use this only for HTTP GET requests! This script does aggressive
# retries, so if you use it for a non-readonly HTTP method that doesn't use any
# idempotency token validation mechanism you might end up with duplicate modifications.
function curl_with_retry {
    with_log curl \
        --location \
        --silent \
        --show-error \
        --fail \
        --retry 5 \
        --retry-all-errors \
        "$@"
}

function move_to_path {
    with_log mv "$1" $HOME/.cargo/bin
}
