let output_dir = ($env.FILE_PWD | path join "generated")
mkdir $output_dir
^openssl version -a | save -f ($output_dir | path join "openssl.txt")
^sysctl -n machdep.cpu.brand_string hw.memsize hw.ncpu | save -f ($output_dir | path join "host.txt")
for trial in 1..3 {
    let result = (^openssl speed -elapsed -seconds 1 ML-DSA-65 ML-DSA-87 SLH-DSA-SHA2-128s SLH-DSA-SHA2-128f SLH-DSA-SHA2-256s SLH-DSA-SHA2-256f | complete)
    $result.stdout | save -f ($output_dir | path join $"trial-($trial).stdout.txt")
    $result.stderr | save -f ($output_dir | path join $"trial-($trial).stderr.txt")
    if $result.exit_code != 0 { error make {msg: $"OpenSSL speed trial ($trial) failed"} }
}
