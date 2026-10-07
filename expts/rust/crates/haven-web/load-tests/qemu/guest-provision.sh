#!/usr/bin/env bash
set -euo pipefail

PUBKEY=${1:?usage: guest-provision.sh "<ssh public key>"}

# This script must be run inside the QEMU Debian Guest as root!
if [ "$EUID" -ne 0 ]; then
  echo "Please run this script inside the guest as root."
  exit 1
fi

PG_MAJOR=18
export DEBIAN_FRONTEND=noninteractive

echo "=== 1. Base Setup & Root SSH ==="
swapoff -a
sed -i '/\sswap\s/d' /etc/fstab

# Remove cdrom sources if installer left them behind
sed -i '/cdrom:/d' /etc/apt/sources.list 2>/dev/null || true
sed -i '/cdrom:/d' /etc/apt/sources.list.d/debian.sources 2>/dev/null || true

apt-get update
apt-get -y full-upgrade
apt-get install -y curl ca-certificates sysstat postgresql-common nginx openssl openssh-server sudo

install -d -m 700 /root/.ssh
echo "$PUBKEY" > /root/.ssh/authorized_keys
chmod 600 /root/.ssh/authorized_keys

# Robustly allow root login (key only)
mkdir -p /etc/ssh/sshd_config.d
echo "PermitRootLogin prohibit-password" > /etc/ssh/sshd_config.d/99-root-login.conf
systemctl restart ssh

# Mask background noise
systemctl mask apt-daily.timer apt-daily-upgrade.timer fstrim.timer logrotate.timer man-db.timer \
               e2scrub_all.timer sysstat-collect.timer sysstat-summary.timer \
               systemd-tmpfiles-clean.timer dpkg-db-backup.timer

echo "=== 2. PostgreSQL Setup ==="
/usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y
apt-get update
apt-get install -y "postgresql-$PG_MAJOR"
apt-mark hold "postgresql-$PG_MAJOR" nginx

mkdir -p /etc/postgresql/$PG_MAJOR/main/conf.d
cat > /etc/postgresql/$PG_MAJOR/main/conf.d/haven.conf <<'EOF2'
shared_buffers = 512MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 128MB
max_connections = 30
checkpoint_completion_target = 0.9
shared_preload_libraries = 'pg_stat_statements'
track_io_timing = on
listen_addresses = '*'
EOF2

systemctl restart postgresql

runuser -u postgres -- psql -tc "SELECT 1 FROM pg_roles WHERE rolname='haven'" | grep -q 1 \
  || runuser -u postgres -- psql -c "CREATE USER haven WITH PASSWORD 'haven';"
runuser -u postgres -- psql -tc "SELECT 1 FROM pg_database WHERE datname='haven'" | grep -q 1 \
  || runuser -u postgres -- psql -c "CREATE DATABASE haven OWNER haven;"
grep -qF "10.10.0.1/32" /etc/postgresql/$PG_MAJOR/main/pg_hba.conf \
  || echo "host all haven 10.10.0.1/32 scram-sha-256" >> /etc/postgresql/$PG_MAJOR/main/pg_hba.conf

systemctl reload postgresql

echo "=== 3. Nginx TLS Cert & Config ==="
mkdir -p /etc/nginx/tls
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -days 365 \
  -keyout /etc/nginx/tls/bench.key -out /etc/nginx/tls/bench.crt \
  -subj "/CN=haven.bench" -addext "subjectAltName=IP:10.10.0.2"
chmod 600 /etc/nginx/tls/bench.key

cat > /etc/nginx/nginx.conf <<'EOF2'
user www-data;
worker_processes 1;
worker_rlimit_nofile 65536;
pid /run/nginx.pid;
error_log /var/log/nginx/error.log warn;
events { worker_connections 4096; }
http {
    include /etc/nginx/mime.types;
    sendfile on; tcp_nopush on;
    access_log /var/log/nginx/access.log combined buffer=64k flush=5s;
    keepalive_timeout 65; keepalive_requests 10000;
    upstream haven { server 127.0.0.1:8080; keepalive 32; }
    server {
        listen 443 ssl; http2 on; server_name _;
        ssl_certificate /etc/nginx/tls/bench.crt;
        ssl_certificate_key /etc/nginx/tls/bench.key;
        ssl_protocols TLSv1.2 TLSv1.3; ssl_session_cache shared:SSL:10m; ssl_session_timeout 1h;
        location / {
            proxy_pass http://haven; proxy_http_version 1.1;
            proxy_set_header Connection ""; proxy_set_header Host $host;
            proxy_set_header X-Forwarded-For $remote_addr; proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
EOF2
nginx -t

echo "=== 4. Application Env & Systemd Unit ==="
useradd --system --home /opt/haven haven || true
mkdir -p /opt/haven /etc/haven
chown haven:haven /opt/haven

cat > /etc/haven/haven.env <<EOF2
DATABASE_URL=postgres://haven:haven@127.0.0.1:5432/haven
RUST_LOG=warn
DB_POOL_SIZE=16
HOST=0.0.0.0
PORT=8080
EOF2

cat > /etc/systemd/system/haven-web.service <<'EOF2'
[Unit]
After=postgresql.service network.target
Wants=postgresql.service

[Service]
User=haven
WorkingDirectory=/opt/haven
EnvironmentFile=/etc/haven/haven.env
ExecStart=/opt/haven/haven-web
Restart=on-failure
RestartSec=2
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF2

echo "=== 5. Network Rename & IP (bench0) ==="
mkdir -p /etc/systemd/network
cat > /etc/systemd/network/10-bench0.link <<EOF2
[Match]
MACAddress=52:54:00:12:34:56
[Link]
Name=bench0
EOF2

cat > /etc/network/interfaces <<EOF2
# interfaces(5) file used by ifup(8) and ifdown(8)
auto lo
iface lo inet loopback

allow-hotplug bench0
iface bench0 inet static
    address 10.10.0.2/24
    gateway 10.10.0.1
EOF2

sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="quiet"/GRUB_CMDLINE_LINUX_DEFAULT="quiet console=ttyS0"/' /etc/default/grub
update-grub
systemctl enable serial-getty@ttyS0 haven-web nginx

echo "=== 6. Record State ==="
dpkg-query -W > /root/packages.txt

echo "=== DONE! ==="
echo "Check the dry run verification steps before relying on the boot."
