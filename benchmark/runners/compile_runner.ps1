param(
    [int]$Iterations = 10
)

$ErrorActionPreference = "Stop"

$ProjectRoot = "D:\Osis\AeroxA"
$Compiler = "D:\cargo-target\release\Aeroxa.exe"
$SuiteDirectory = Join-Path $ProjectRoot "benchmark\suites"
$ResultDirectory = Join-Path $ProjectRoot "benchmark\results"
$TempDirectory = Join-Path $ProjectRoot "benchmark\temporary"

# Validate paths.
if (!(Test-Path $Compiler)) {
    throw "AEROXA compiler not found: $Compiler"
}

if (!(Test-Path $SuiteDirectory)) {
    throw "Benchmark suites directory not found."
}

New-Item -ItemType Directory -Force -Path $ResultDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $TempDirectory | Out-Null

$Results = @()

$Sources = Get-ChildItem `
    -Path $SuiteDirectory `
    -Filter "*.ex" `
    -Recurse `
    -File

if ($Sources.Count -eq 0) {
    Write-Host "No AEROXA benchmark source files found." -ForegroundColor Yellow
    exit
}

Write-Host ""
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "       AEROXA BENCHMARK SUITE" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Iterations: $Iterations"
Write-Host "Sources: $($Sources.Count)"
Write-Host ""

foreach ($Source in $Sources) {

    Write-Host "Benchmark: $($Source.Name)" -ForegroundColor Cyan

    for ($i = 1; $i -le $Iterations; $i++) {

        # Use a unique source path per iteration
        # to avoid reusing the same source cache.
        $TempSource = Join-Path `
            $TempDirectory `
            "$($Source.BaseName)_$i.ex"

        Copy-Item $Source.FullName $TempSource -Force

        $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

        try {
            $Output = & $Compiler $TempSource 2>&1
            $ExitCode = $LASTEXITCODE

            $Stopwatch.Stop()

            if ($ExitCode -ne 0) {
                Write-Host "  FAILED iteration $i" -ForegroundColor Red
                $Output | ForEach-Object {
                    Write-Host $_
                }

                continue
            }

            $Elapsed = [math]::Round(
                $Stopwatch.Elapsed.TotalMilliseconds,
                3
            )

            Write-Host "  Run $i : $Elapsed ms" -ForegroundColor Green

            $Results += [PSCustomObject]@{
                Benchmark = $Source.BaseName
                Iteration = $i
                PipelineMilliseconds = $Elapsed
                Compiler = "AEROXA"
                Configuration = "Release"
            }
        }
        finally {
            Remove-Item $TempSource -Force -ErrorAction SilentlyContinue
        }
    }
}

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"

$CsvPath = Join-Path `
    $ResultDirectory `
    "aeroxa_compile_pipeline_$Timestamp.csv"

$Results | Export-Csv `
    -Path $CsvPath `
    -NoTypeInformation `
    -Encoding UTF8

Write-Host ""
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "       BENCHMARK COMPLETED" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Cyan

Write-Host "Results saved to:"
Write-Host $CsvPath -ForegroundColor Yellow