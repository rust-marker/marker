. $(dirname ${BASH_SOURCE[0]})/lib.sh

# Replaces text using a sed pattern in the region of the file surrounded by
# `region replace {region}` and `endregion replace {region}` comments.
function replace_in_regions_for_file {
    local file="$1"
    local region="replace $2"
    local sed_pattern="$3"

    local comment_begin="(#|\/\/|<!--)"
    local comment_end="( -->)?$"

    # Replace both the version itself, and the sliding tags
    #
    # There is a caveat here for the major version. It is rather ambiguous, because
    # it is just a single number, there are no dots in it that could identify it as
    # semver version. So for the major version we require that it is always specified
    # with the `v` prefix e.g. `v1`.
    with_log sed --regexp-extended --follow-symlinks --in-place --file - "$file" <<EOF
        /$comment_begin region $region$comment_end/,/$comment_begin endregion $region$comment_end/ \
        {
            $sed_pattern
        }
EOF
}

# Applies a sed replacement pattern within the desired regions to all files in the repo
function replace_in_regions {
    local file_patterns=(
        .
        :!:scripts/release/set-version.diff
    )

    for file in $(with_log git ls-files -- "${file_patterns[@]}"); do
        replace_in_regions_for_file "$file" "$@"
    done
}

# Replaces semver patterns `X.Y.X(-suffix)?`, `X.Y` and `vX` within the desired
# regions in all files in the repo.
#
# There is a caveat here for the major version. It is rather ambiguous, because
# it is just a single number, there are no dots in it that could identify it as
# semver version. So for the major version we require that it is always specified
# with the `v` prefix e.g. `v1`.
function replace_semver_in_regions {
    local region="$1"
    local x_y_z="$2"

    local suffix='(-[0-9a-zA-Z.\-]+)?'
    local num='[0-9]+'
    local pattern="$num\.$num\.$num$suffix"

    if ! [[ "$x_y_z" =~ ^$pattern$ ]]; then
        die "Please enter a valid semver version like '1.2.3'. Got '$x_y_z'."
    fi

    local x_y=$(echo "$x_y_z" | cut --delimiter . --fields 1-2)
    local x=$(echo "$x_y_z" | cut --delimiter . --fields 1)

    replace_in_regions "$region" "
        s/(v|\W)$pattern/\1$x_y_z/g
        s/(v|\W)$num\.$num$suffix/\1$x_y/g
        s/v$num$suffix/v$x/g
    "
}

# Replaces date patterns `YYYY-MM-DD` within the desired regions in all files in the repo.
function replace_date_in_regions {
    local region="$1"
    local date="$2"

    local d='[0-9]'
    local pattern="$d{4}-$d{2}-$d{2}"

    if ! [[ "$date" =~ ^$pattern$ ]]; then
        die "Please enter a valid date like '2022-01-01'. Got '$date'."
    fi

    replace_in_regions "$region" "s/$pattern/$date/g"
}
