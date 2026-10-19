# The static site (dist/, copied to ./site by deploy.sh) served by nginx on Cloud Run, behind a
# username + password (htpasswd made at deploy time, never committed).
FROM nginx:1.27-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY site/ /usr/share/nginx/html/
