param([int]$Iterations = 10)

$ErrorActionPreference = "Stop"

$Root = "D:\Osis\AeroxA"
$AeroXa = "D:\cargo-target\release\Aeroxa.exe"
$Rustc = (Get-Command rustc).Source
$Clang = (Get-Command clang++).Source

$Temp = "$Root\benchmark\temporary\cross_language"
$ResultsDir = "$Root\benchmark\results"

New-Item -ItemType Directory -Force $Temp,$ResultsDir | Out-Null

if (!(Test-Path $AeroXa)) { throw "AEROXA not found" }

if ($Iterations -lt 1) {
    throw "Iterations must be at least 1."
}

$Results = [System.Collections.Generic.List[object]]::new()

function Measure-Compile($Name,$Source,$Output) {
    $sw = [Diagnostics.Stopwatch]::StartNew()

    if ($Name -eq "AEROXA") {
        & $AeroXa $Source --compile-only 2>&1 | Out-Null
    }
    elseif ($Name -eq "Rust") {
        & $Rustc -O $Source -o $Output 2>&1 | Out-Null
    }
    else {
        & $Clang -O2 $Source -o $Output 2>&1 | Out-Null
    }

    $exit = $LASTEXITCODE
    $sw.Stop()

    if ($exit -ne 0) {
        throw "$Name compilation failed."
    }

    return [math]::Round($sw.Elapsed.TotalMilliseconds,3)
}

$Sources = Get-ChildItem "$Root\benchmark\suites" -Filter "*.ex" -Recurse -File

foreach ($src in $Sources) {
    $base = $src.BaseName
    $category = Split-Path $src.DirectoryName -Leaf
    $dir = $src.DirectoryName

    $files = @(
        @{Name="AEROXA"; Path=$src.FullName; Ext=".ex"},
        @{Name="Rust"; Path="$dir\$base.rs"; Ext=".rs"},
        @{Name="C++"; Path="$dir\$base.cpp"; Ext=".cpp"}
    )

    foreach ($lang in $files) {
        if (!(Test-Path $lang.Path)) {
            throw "Missing source: $($lang.Path)"
        }
    }

    Write-Host "`nCATEGORY: $category" -ForegroundColor Cyan

    # WARM-UP
    Write-Host "Warm-up: 3 runs per compiler" -ForegroundColor DarkGray

    foreach ($lang in ($files | Get-Random -Count $files.Count)) {
        Write-Host "[$($lang.Name)]" -ForegroundColor Yellow

        for ($w = 1; $w -le 3; $w++) {
            $id = [guid]::NewGuid().ToString("N")
            $tempSource = "$Temp\warmup_$base`_$id$($lang.Ext)"
            $output = "$Temp\warmup_$base`_$id.exe"

            Copy-Item $lang.Path $tempSource

            try {
                Measure-Compile $lang.Name $tempSource $output | Out-Null
            }
            finally {
                Remove-Item $tempSource,$output -Force -ErrorAction SilentlyContinue
            }
        }
    }

    # MEASUREMENT
    Write-Host "Measurement: $Iterations runs" -ForegroundColor Cyan

    for ($i = 1; $i -le $Iterations; $i++) {

        # RANDOMIZE COMPILER ORDER EACH ROUND
        $round = $files | Get-Random -Count $files.Count

        foreach ($lang in $round) {
            $id = [guid]::NewGuid().ToString("N")
            $tempSource = "$Temp\$base`_$id$($lang.Ext)"
            $output = "$Temp\$base`_$id.exe"

            Copy-Item $lang.Path $tempSource

            try {
                $time = Measure-Compile $lang.Name $tempSource $output

                Write-Host "Run $i [$($lang.Name)] : $time ms" -ForegroundColor Green

                $Results.Add([PSCustomObject]@{
                    Category = $category
                    Compiler = $lang.Name
                    Iteration = $i
                    CompileMilliseconds = $time
                })
            }
            finally {
                Remove-Item $tempSource,$output -Force -ErrorAction SilentlyContinue
            }
        }
    }
}

$runId = Get-Date -Format "yyyyMMdd_HHmmss"
$csv = "$ResultsDir\cross_language_$runId.csv"

$Results | Export-Csv $csv -NoTypeInformation -Encoding UTF8

# SUMMARY
$Summary = foreach ($group in ($Results | Group-Object Category,Compiler)) {
    $times = @($group.Group.CompileMilliseconds | Sort-Object)
    $n = $times.Count

    if ($n -eq 0) { continue }

    if ($n % 2 -eq 0) {
        $median = ($times[$n/2-1] + $times[$n/2]) / 2
    }
    else {
        $median = $times[[math]::Floor($n/2)]
    }

    [PSCustomObject]@{
        Category = $group.Group[0].Category
        Compiler = $group.Group[0].Compiler
        MedianMs = [math]::Round($median,3)
        MinimumMs = $times[0]
        MaximumMs = $times[-1]
        Iterations = $n
    }
}

Write-Host "`n========== AEROXA BENCHMARK REPORT ==========" -ForegroundColor Cyan

$Summary | Sort-Object Category,Compiler | Format-Table -AutoSize

Write-Host "CSV: $csv" -ForegroundColor Green
Write-Host "BENCHMARK COMPLETED" -ForegroundColor Green