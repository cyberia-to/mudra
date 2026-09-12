let bench_dir = $env.FILE_PWD
let rows = (1..3 | each {|trial|
    open --raw ($bench_dir | path join $"trial-($trial).stdout.txt")
    | lines
    | parse --regex '^\s*(?<algorithm>\S+)\s+(?<keygen>[0-9.]+)s\s+(?<sign>[0-9.]+)s\s+(?<verify>[0-9.]+)s\s+(?<keygen_rate>[0-9.]+)\s+(?<sign_rate>[0-9.]+)\s+(?<verify_rate>[0-9.]+)\s*$'
    | each {|row|
        {
            algorithm: $row.algorithm
            trial: $trial
            keygen_ms: (1000.0 / ($row.keygen_rate | into float))
            sign_ms: (1000.0 / ($row.sign_rate | into float))
            verify_ms: (1000.0 / ($row.verify_rate | into float))
        }
    }
} | flatten)
if ($rows | length) != 18 { error make {msg: "Expected 6 algorithms in each of 3 trials"} }
$rows | to csv | save -f ($bench_dir | path join "samples.csv")
let summary = ($rows | group-by algorithm | transpose algorithm samples | each {|group|
    {
        algorithm: $group.algorithm
        keygen_ms: ($group.samples.keygen_ms | math median)
        sign_ms: ($group.samples.sign_ms | math median)
        sign_min_ms: ($group.samples.sign_ms | math min)
        sign_max_ms: ($group.samples.sign_ms | math max)
        verify_ms: ($group.samples.verify_ms | math median)
    }
})
$summary | to csv | save -f ($bench_dir | path join "summary.csv")
$summary
