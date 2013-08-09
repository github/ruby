dir="$(dirname "$0")"
"$dir/ruby" -I "$dir/lib" -I "$dir/.ext/common" -I "$dir/.ext/x86_64-darwin12.3.0" -I "$dir" "$@"
