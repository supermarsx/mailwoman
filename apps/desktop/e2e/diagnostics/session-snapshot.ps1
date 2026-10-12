# Prints what the desktop shell looks like to the OS while a WebDriver session is being
# created. wdio.conf.mjs runs this once during the session wait and prints the result only
# when the run fails (see README, "When the session does not open").
#
# It answers three questions the driver's own error cannot:
#   * is this process tree elevated (WebView2 Runtime 150+ then ignores the WEBVIEW2_*
#     environment variables msedgedriver relies on),
#   * did the shell start a WebView2 browser process, with which user data folder, and
#     does it carry --remote-debugging-port,
#   * is the shell sitting on a dialog (its text is printed).
#
# Windows PowerShell 5.1 compatible. Read-only: it changes nothing.
param([Parameter(Mandatory = $true)][string]$ExeName)

$ErrorActionPreference = 'Continue'

Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class MwSessionSnapshot {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr parent, EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  static string Text(IntPtr h) { var s = new StringBuilder(4096); GetWindowText(h, s, s.Capacity); return s.ToString(); }
  static string Class(IntPtr h) { var s = new StringBuilder(256); GetClassName(h, s, s.Capacity); return s.ToString(); }
  public static List<string> Windows(uint wanted) {
    var lines = new List<string>();
    EnumWindows((h, l) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid != wanted || !IsWindowVisible(h)) return true;
      lines.Add(String.Format("window class={0} title=\"{1}\"", Class(h), Text(h)));
      EnumChildWindows(h, (c, l2) => {
        var t = Text(c);
        if (t.Length > 0) lines.Add(String.Format("    {0}: {1}", Class(c), t.Replace("\r", "").Replace("\n", " | ")));
        return true;
      }, IntPtr.Zero);
      return true;
    }, IntPtr.Zero);
    return lines;
  }
}
'@

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$elevated = ([Security.Principal.WindowsPrincipal]$identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
"elevated: $elevated (user $($identity.Name))"

$guid = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
foreach ($key in @(
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\$guid",
    "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\$guid",
    "HKCU:\Software\Microsoft\EdgeUpdate\Clients\$guid")) {
  $pv = $null
  try { $pv = (Get-ItemProperty -Path $key -Name pv -ErrorAction Stop).pv } catch {}
  if ($pv) { "WebView2 Evergreen runtime: $pv ($key)" }
}
if ($env:WEBVIEW2_BROWSER_EXECUTABLE_FOLDER) { "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER=$env:WEBVIEW2_BROWSER_EXECUTABLE_FOLDER" }
foreach ($root in 'HKLM', 'HKCU') {
  $policy = "${root}:\SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments"
  if (Test-Path $policy) {
    $values = Get-ItemProperty -Path $policy
    foreach ($name in @($ExeName, '*')) {
      if ($values.PSObject.Properties.Name -contains $name) { "policy $policy `"$name`" = $($values.$name)" }
    }
  }
}

$shells = @(Get-CimInstance Win32_Process -Filter "Name='$ExeName'")
"shell processes ($ExeName): $($shells.Count)"
foreach ($p in $shells) {
  "  pid $($p.ProcessId) parent $($p.ParentProcessId) started $($p.CreationDate.ToString('HH:mm:ss'))"
  foreach ($line in [MwSessionSnapshot]::Windows([uint32]$p.ProcessId)) { "    $line" }
}

# Only the WebView2 processes this shell started; other apps' WebView2 processes are not listed.
$mine = @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" |
    Where-Object { $_.CommandLine -like "*--webview-exe-name=$ExeName*" })
$browsers = @($mine | Where-Object { $_.CommandLine -notlike '*--type=*' })
"WebView2 browser processes started by the shell: $($browsers.Count) (+$($mine.Count - $browsers.Count) child processes)"
foreach ($b in $browsers) {
  $dir = '(none)'
  $m = [regex]::Match($b.CommandLine, '--user-data-dir="([^"]*)"')
  if ($m.Success) { $dir = $m.Groups[1].Value }
  $debug = $b.CommandLine -like '*--remote-debugging-port=*'
  "  pid $($b.ProcessId) parent $($b.ParentProcessId)"
  "    user data folder: $dir"
  "    --remote-debugging-port on its command line: $debug"
  "    DevToolsActivePort file present: $(Test-Path (Join-Path $dir 'DevToolsActivePort'))"
}
