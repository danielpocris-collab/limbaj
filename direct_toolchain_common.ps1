function Resolve-WorkspacePath {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }

    return [System.IO.Path]::GetFullPath((Join-Path $Workspace $Path))
}

function Reset-BuildDirectory {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $workspaceRoot = [System.IO.Path]::GetFullPath($Workspace)
    if (-not $workspaceRoot.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $workspaceRoot += [System.IO.Path]::DirectorySeparatorChar
    }

    $targetPath = [System.IO.Path]::GetFullPath($Path)
    if (-not $targetPath.StartsWith($workspaceRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "refusing to reset directory outside workspace: $targetPath"
    }

    if (Test-Path -LiteralPath $targetPath) {
        Remove-Item -LiteralPath $targetPath -Recurse -Force
    }

    New-Item -ItemType Directory -Force -Path $targetPath | Out-Null
    return $targetPath
}

function Read-FileSafe {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return ""
    }

    return Get-Content -LiteralPath $Path -Raw
}

function New-DetachedProcessLogs {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Tag,
        [Parameter(Mandatory = $true)][string]$StdoutName,
        [Parameter(Mandatory = $true)][string]$StderrName
    )

    # Keep redirected stderr outside the process working directory; some large
    # native selfhost builds crash when stderr is redirected into the cwd.
    $safeTag = $Tag -replace '[^A-Za-z0-9._-]', '_'
    $logsRoot = Join-Path $Workspace "_tool_logs"
    $logDir = Reset-BuildDirectory -Workspace $Workspace -Path (Join-Path $logsRoot $safeTag)

    return [PSCustomObject]@{
        StdoutPath = Join-Path $logDir $StdoutName
        StderrPath = Join-Path $logDir $StderrName
    }
}

function New-CaseFailureDetail {
    param(
        [Parameter(Mandatory = $true)][string]$Reason,
        [Parameter(Mandatory = $true)][string]$CompileStdout,
        [Parameter(Mandatory = $true)][string]$CompileStderr,
        [Parameter(Mandatory = $true)][string]$RunStdout,
        [Parameter(Mandatory = $true)][string]$RunStderr
    )

    return @(
        $Reason
        "== compile stdout =="
        $CompileStdout
        "== compile stderr =="
        $CompileStderr
        "== run stdout =="
        $RunStdout
        "== run stderr =="
        $RunStderr
    ) -join [Environment]::NewLine
}

function Resolve-CompilerExecutable {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Compiler,
        [Parameter(Mandatory = $true)][string]$BuildTag
    )

    $compilerPath = Resolve-WorkspacePath -Workspace $Workspace -Path $Compiler
    if (-not (Test-Path -LiteralPath $compilerPath)) {
        throw "missing compiler target: $compilerPath"
    }

    $extension = [System.IO.Path]::GetExtension($compilerPath)
    if ($extension -ieq ".exe") {
        return $compilerPath
    }

    if ($extension -ine ".ng") {
        throw "unsupported compiler target: $compilerPath"
    }

    $trustedCompiler = Resolve-WorkspacePath -Workspace $Workspace -Path "ng_selfhost_clean.exe"
    if (-not (Test-Path -LiteralPath $trustedCompiler)) {
        throw "missing trusted selfhost checkpoint: $trustedCompiler"
    }

    $buildDir = Reset-BuildDirectory -Workspace $Workspace -Path (Join-Path $Workspace $BuildTag)
    $outputPath = Join-Path $buildDir "output.exe"
    $targetExe = Join-Path $buildDir "$BuildTag.exe"
    $logs = New-DetachedProcessLogs -Workspace $Workspace -Tag "${BuildTag}_bootstrap" -StdoutName "compile_stdout.txt" -StderrName "compile_stderr.txt"
    $stdoutPath = $logs.StdoutPath
    $stderrPath = $logs.StderrPath

    $compile = Start-Process -FilePath $trustedCompiler `
        -ArgumentList @($compilerPath) `
        -WorkingDirectory $buildDir `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath

    if ($compile.ExitCode -ne 0) {
        $stdout = Read-FileSafe -Path $stdoutPath
        $stderr = Read-FileSafe -Path $stderrPath
        throw "failed to bootstrap compiler source $compilerPath with $trustedCompiler`n== compile stdout ==`n$stdout`n== compile stderr ==`n$stderr"
    }

    if (-not (Test-Path -LiteralPath $outputPath)) {
        throw "bootstrap compile did not produce $outputPath"
    }

    Copy-Item -LiteralPath $outputPath -Destination $targetExe -Force
    return $targetExe
}

function Invoke-NativeCompilerBuild {
    param(
        [Parameter(Mandatory = $true)][string]$CompilerExe,
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Program,
        [Parameter(Mandatory = $true)][string]$OutDir
    )

    $programPath = Resolve-WorkspacePath -Workspace $Workspace -Path $Program
    if (-not (Test-Path -LiteralPath $programPath)) {
        throw "missing input program: $programPath"
    }

    $OutDir = Reset-BuildDirectory -Workspace $Workspace -Path $OutDir
    $outputPath = Join-Path $OutDir "output.exe"
    $outLeaf = [System.IO.Path]::GetFileName($OutDir)
    $logs = New-DetachedProcessLogs -Workspace $Workspace -Tag "${outLeaf}_compile" -StdoutName "compile_stdout.txt" -StderrName "compile_stderr.txt"
    $stdoutPath = $logs.StdoutPath
    $stderrPath = $logs.StderrPath

    $compile = Start-Process -FilePath $CompilerExe `
        -ArgumentList @($programPath) `
        -WorkingDirectory $OutDir `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath

    if ($compile.ExitCode -ne 0) {
        $stdout = Read-FileSafe -Path $stdoutPath
        $stderr = Read-FileSafe -Path $stderrPath
        throw "native compile failed for $programPath with exit $($compile.ExitCode)`n== compile stdout ==`n$stdout`n== compile stderr ==`n$stderr"
    }

    if (-not (Test-Path -LiteralPath $outputPath)) {
        throw "native compile did not produce $outputPath"
    }

    return $outputPath
}

function Read-RuntimeCorpusCases {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [string]$Manifest = "tests/programs/runtime_corpus_manifest.txt"
    )

    $manifestPath = Resolve-WorkspacePath -Workspace $Workspace -Path $Manifest
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        throw "missing runtime corpus manifest: $manifestPath"
    }

    $cases = @()
    foreach ($rawLine in Get-Content -LiteralPath $manifestPath) {
        $line = $rawLine.Trim()
        if ($line.Length -eq 0) {
            continue
        }
        if ($line.StartsWith("#")) {
            continue
        }

        $parts = $line.Split("|")
        if ($parts.Count -ne 6) {
            throw "invalid runtime corpus manifest line: $line"
        }

        $runArgs = @()
        if ($parts[3].Length -gt 0) {
            $runArgs = @($parts[3].Split(";") | Where-Object { $_.Length -gt 0 })
        }

        $expectedFile = $null
        if ($parts[4].Length -gt 0) {
            $expectedFile = $parts[4]
        }

        $expectedFileSize = $null
        if ($parts[5].Length -gt 0) {
            $expectedFileSize = [int64]$parts[5]
        }

        $cases += [PSCustomObject]@{
            Name = $parts[0]
            Program = $parts[1]
            ExpectedExit = [int]$parts[2]
            RunArgs = $runArgs
            ExpectedFile = $expectedFile
            ExpectedFileSize = $expectedFileSize
        }
    }

    return $cases
}

function Invoke-RuntimeCorpusCase {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$CompilerExe,
        [Parameter(Mandatory = $true)][object]$Case,
        [Parameter(Mandatory = $true)][string]$CaseDir
    )

    $outputPath = Join-Path $CaseDir "output.exe"
    $caseLeaf = [System.IO.Path]::GetFileName($CaseDir)
    $compileLogs = New-DetachedProcessLogs -Workspace $Workspace -Tag "${caseLeaf}_compile" -StdoutName "host_stdout.txt" -StderrName "host_stderr.txt"
    $runLogs = New-DetachedProcessLogs -Workspace $Workspace -Tag "${caseLeaf}_run" -StdoutName "run_stdout.txt" -StderrName "run_stderr.txt"
    $compileStdoutPath = $compileLogs.StdoutPath
    $compileStderrPath = $compileLogs.StderrPath
    $runStdoutPath = $runLogs.StdoutPath
    $runStderrPath = $runLogs.StderrPath

    New-Item -ItemType Directory -Force -Path $CaseDir | Out-Null
    Remove-Item -LiteralPath $outputPath -Force -ErrorAction SilentlyContinue

    if ($Case.RunArgs -contains "test_input.txt") {
        Copy-Item -LiteralPath (Join-Path $Workspace "test_input.txt") -Destination (Join-Path $CaseDir "test_input.txt") -Force
    }

    if ($null -ne $Case.ExpectedFile -and $Case.ExpectedFile.Length -gt 0) {
        Remove-Item -LiteralPath (Join-Path $CaseDir $Case.ExpectedFile) -Force -ErrorAction SilentlyContinue
    }

    try {
        $programPath = Resolve-WorkspacePath -Workspace $Workspace -Path $Case.Program
        $compile = Start-Process -FilePath $CompilerExe `
            -ArgumentList @($programPath) `
            -WorkingDirectory $CaseDir `
            -NoNewWindow `
            -Wait `
            -PassThru `
            -RedirectStandardOutput $compileStdoutPath `
            -RedirectStandardError $compileStderrPath

        if ($compile.ExitCode -ne 0) {
            throw "compile failed with exit $($compile.ExitCode)"
        }

        if (-not (Test-Path -LiteralPath $outputPath)) {
            throw "output.exe missing after compile"
        }

        if ($Case.RunArgs.Count -gt 0) {
            $run = Start-Process -FilePath $outputPath `
                -ArgumentList $Case.RunArgs `
                -WorkingDirectory $CaseDir `
                -NoNewWindow `
                -Wait `
                -PassThru `
                -RedirectStandardOutput $runStdoutPath `
                -RedirectStandardError $runStderrPath
        } else {
            $run = Start-Process -FilePath $outputPath `
                -WorkingDirectory $CaseDir `
                -NoNewWindow `
                -Wait `
                -PassThru `
                -RedirectStandardOutput $runStdoutPath `
                -RedirectStandardError $runStderrPath
        }

        if ($run.ExitCode -ne [int]$Case.ExpectedExit) {
            throw "runtime exit mismatch: expected $($Case.ExpectedExit), got $($run.ExitCode)"
        }

        if ($null -ne $Case.ExpectedFile -and $Case.ExpectedFile.Length -gt 0) {
            $expectedFilePath = Join-Path $CaseDir $Case.ExpectedFile
            if (-not (Test-Path -LiteralPath $expectedFilePath)) {
                throw "expected file missing: $($Case.ExpectedFile)"
            }
            $actualSize = (Get-Item -LiteralPath $expectedFilePath).Length
            if ($actualSize -ne [int64]$Case.ExpectedFileSize) {
                throw "expected file size $($Case.ExpectedFileSize), got $actualSize"
            }
        }

        return [PSCustomObject]@{
            Name = $Case.Name
            Ok = $true
            Detail = "ok"
        }
    } catch {
        return [PSCustomObject]@{
            Name = $Case.Name
            Ok = $false
            Detail = New-CaseFailureDetail `
                -Reason $_.Exception.Message `
                -CompileStdout (Read-FileSafe -Path $compileStdoutPath) `
                -CompileStderr (Read-FileSafe -Path $compileStderrPath) `
                -RunStdout (Read-FileSafe -Path $runStdoutPath) `
                -RunStderr (Read-FileSafe -Path $runStderrPath)
        }
    }
}
