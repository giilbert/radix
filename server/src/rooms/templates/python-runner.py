# RADIX TEST STUFF -- DO NOT TOUCH

import json, time

__RADIX_TEST_INPUTS = json.loads("{{INPUTS}}")
output = []


start_time = time.time_ns() // 1_000_000

for input in __RADIX_TEST_INPUTS:
    output.append(solve(*input))

end_time = time.time_ns() // 1_000_000

print(
    "[[RADIX TEST OUTPUT]]",
    json.dumps(
        {
            "runtime": end_time - start_time,
            "program_output": output,
        },
        separators=(",", ":"),
    ),
)
