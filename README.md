# crab-gateway
try to produce a proxy
git config --global http.proxy socks5://192.168.1.3:7890
git config --global https.proxy socks5://192.168.1.3:7890

# http body 读取
- http1.0 
    没有CONTENT-LENGTH，只能持续读取。
- CHUNK
    数据分块传输。