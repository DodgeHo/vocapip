$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$python = Join-Path $workspace "content-pipeline\.venv\Scripts\python.exe"
if (-not (Test-Path -LiteralPath $python)) { $python = "python" }
& $python -X utf8 (Join-Path $workspace "content-pipeline\src\vocabularybook_pipeline.py") validate
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $python -X utf8 (Join-Path $workspace "content-pipeline\src\vocabularybook_pipeline.py") rebuild-check
exit $LASTEXITCODE
