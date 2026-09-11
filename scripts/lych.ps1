param(
    [Parameter(Position = 0)]
    [string]$Command
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Project = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Write-Host "Project root: $Project" -ForegroundColor DarkGray

switch ($Command) {
    'build' {
        cargo build --release
    }

    'run' {
        cargo build --release

        & qemu-system-aarch64 `
            -M virt `
            -cpu cortex-a72 `
            -nographic `
            -kernel $Project\target\aarch64-unknown-none\release\kernel
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    'debug' {
        cargo build --release

        & qemu-system-aarch64 `
            -M virt `
            -cpu cortex-a72 `
            -nographic `
            -S `
            -gdb tcp::1234 `
            -kernel $Project\target\aarch64-unknown-none\release\kernel
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    'gdb' {
        & gdb-multiarch `
            $Project\target\aarch64-unknown-none\release\kernel `
            -ex 'target remote :1234'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    'clean' {
        cargo clean
    }

    'fmt' {
        cargo fmt --all
    }

    'fmt-check' {
        cargo fmt --all --check
    }

    default {
        Write-Host 'Usage:' -ForegroundColor Yellow
        Write-Host '  .\scripts\lych.ps1 build'
        Write-Host '  .\scripts\lych.ps1 run'
        Write-Host '  .\scripts\lych.ps1 debug'
        Write-Host '  .\scripts\lych.ps1 gdb'
        Write-Host '  .\scripts\lych.ps1 clean'
        Write-Host '  .\scripts\lych.ps1 fmt'
        Write-Host '  .\scripts\lych.ps1 fmt-check'
        exit 1
    }
}