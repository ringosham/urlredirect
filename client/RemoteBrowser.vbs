' Place this in C:\

' Change this port if you change it in the server side
Set port = 10080

Set objShell = WScript.CreateObject("WScript.Shell")
Set objWMIService = GetObject("winmgmts:\\.\root\cimv2")

' Find default gateway
Set colNetAdapters = objWMIService.ExecQuery("Select DefaultIPGateway From Win32_NetworkAdapterConfiguration Where IPEnabled = True")

hostIP = ""
For Each objAdapter in colNetAdapters
    If Not IsNull(objAdapter.DefaultIPGateway) Then
        ' Arrays in WMI, grab the first gateway IP
        hostIP = objAdapter.DefaultIPGateway(0)
        Exit For
    End If
Next

If hostIP <> "" And WScript.Arguments.Count > 0 Then
    url = WScript.Arguments(0)
    
    command = "curl.exe -m 2 ""http://" & hostIP & ":" & port & "/?l=" & url & """"
    objShell.Run command, 0, False
End If