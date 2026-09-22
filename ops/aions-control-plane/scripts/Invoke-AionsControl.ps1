[CmdletBinding()]
param(
    [Parameter(Mandatory)] [ValidateSet('start','stop','restart','status')] [string]$Action,
    [Parameter(Mandatory)] [string]$ModuleId,
    [Parameter(Mandatory)] [string]$Repo,
    [string]$Branch = 'Darkstar',
    [string]$Reason = 'operator request'
)

$ErrorActionPreference = 'Stop'

$requestId = [guid]::NewGuid().ToString()
$startedAt = (Get-Date).ToUniversalTime().ToString('o')

Write-Host "AIONS CONTROL PLANE" -ForegroundColor Cyan
Write-Host "request_id : $requestId"
Write-Host "action     : $Action"
Write-Host "module     : $ModuleId"
Write-Host "repo       : $Repo"
Write-Host "branch     : $Branch"
Write-Host "reason     : $Reason"

# This bridge intentionally dispatches intent only.
# Darkstar remains responsible for capability and policy authorization.
$payload = [ordered]@{
    request_id  = $requestId
    action      = $Action
    module_id   = $ModuleId
    principal_id = 'windows:operator'
    repo        = $Repo
    branch      = $Branch
    reason      = $Reason
    started_at  = $startedAt
}

$payload | ConvertTo-Json -Depth 5
