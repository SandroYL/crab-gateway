# Gateway-Error
用于处理此项目内各种Error
## 功能
- 允许自定义新的错误类型。需要提供错误名，错误来源，上游错误（可选），错误描述（可选）。
- 当发生错误时，需要满足：能展示出错误链的层级关系，并追溯到最上层。每一层的错误原因都必须展示。例如：BannedError:此链接被拒绝-->NetError(505):此连接断开。

get-netadapter -IncludeHidden | Format-Table -AutoSize | Where-Object Name -like "vEthernet (WSL (Hyper-V firewall))" | Get-NetAdapterAdvancedProperty -IncludeHidden | Where-Object DisplayName -like "Large Send Offload Version 2*" | Set-NetAdapterAdvancedProperty -DisplayValue Disabled