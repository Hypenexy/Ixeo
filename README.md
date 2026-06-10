# Ixeo
### Search the web here:
### https://ixeo.midelight.net  
  
My attempt at the fastest search engine you have ever tried.  
The server might be very underpowered but my hopes and dreams are overpowered.  
  

### Legacy code (Jul 7, 2023)
All of my different scrapers are included in the repository.  
If you wanna see my timeline of learning and laugh at stupid  
code check out the legacy tag or branch!  
  
### Datasets

I have no idea why I have wanted to include my old datasets in a github repo.  
But the newer scraping I will try and create a web server serving those big files I've collected!    
  
I am currently scraping with my 5 nodes right now as of writing,  
there's a lot of work to do but I believe this time it will be a lot better!

### Installation instructions for Linux  
I recommend running the system on separate containers, but you can possibly run it in one  
Debian 12 is perfect.  
You need Rust  
A postgres database  
  
    installation instructions

To install them as services


```bash
nano /etc/systemd/system/ixeo-crawler.service
```
```ini
[Unit]
Description=Ixeo Indexer Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/Ixeo/web-crawler
ExecStart=/root/Ixeo/web-crawler/target/release/web-crawler
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
nano /etc/systemd/system/ixeo-indexer.service
```
```ini
[Unit]
Description=Ixeo Indexer Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/Ixeo/indexer
ExecStart=/root/Ixeo/indexer/target/release/indexer
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
nano /etc/systemd/system/ixeo-server.service
```
```ini
[Unit]
Description=Ixeo HTTP Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/Ixeo/http-server
ExecStart=/root/Ixeo/http-server/target/release/http-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```
  
And to install the services:
```bash
systemctl daemon-reload
```
```bash
systemctl enable ixeo-server.service
systemctl enable ixeo-indexer.service
```
```bash
systemctl start ixeo-server.service
systemctl start ixeo-indexer.service
```

To check the status of the services:
```bash
systemctl status ixeo-server.service
```
Live log:
```bash
journalctl -u ixeo-server.service -f
```
