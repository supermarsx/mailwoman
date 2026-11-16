# Scanner-path catalog

Research reviewed 16 November 2026. No fixed list can cover every custom CMS or
renamed administration route. These lists cover recognizable foreign application
namespaces, management endpoints, exploit targets and sensitive-file exposures
on a Mailwoman origin. They are not a general-purpose reverse-proxy WAF for all
other applications hosted on the same domain.

## Coverage

| Family | Examples |
| --- | --- |
| CMS | WordPress, Joomla, Drupal, TYPO3, Umbraco, Bitrix, Magento, PrestaShop, OpenCart, Concrete, Craft, Ghost, Strapi, Directus, Cockpit, Contao, Textpattern, MODX, Silverstripe, Bolt, ExpressionEngine, CMS Made Simple |
| Webmail and mail administration | Roundcube, RainLoop, SnappyMail, SquirrelMail, Horde, iRedAdmin, PostfixAdmin, ViMbAdmin, Exchange ECP/OWA and Zimbra administration |
| Hosting and server panels | cPanel/WHM, Webmin/Virtualmin/Usermin, Plesk, DirectAdmin, ISPConfig/ISPmanager, Vesta/Hestia, Ajenti, CyberPanel, CWP, Froxlor |
| Application servers and frameworks | Tomcat Manager/Host Manager, JBoss JMX/Web Console, WebLogic, Jolokia, Solr, H2, Jenkins/Hudson, sensitive Spring Actuator endpoints, Druid, NiFi, GeoServer, Laravel Telescope/Ignition, Symfony/Yii debug routes |
| Data, monitoring and infrastructure | phpMyAdmin/phpPgAdmin/Adminer, SQLite/Redis/Mongo tools, Zabbix, Nagios, Icinga, Grafana/Kibana, Prometheus/Alertmanager, RabbitMQ, Portainer, Rancher, Consul, Vault, MinIO, Kubernetes dashboard, Harbor, Nexus/Artifactory, GitLab/Gitea/Gogs, TeamCity/Bamboo/SonarQube, Guacamole |
| Appliances and exposed files | Fortinet/Pulse/Palo Alto/Cisco VPN namespaces, router CGI/management endpoints, VMware SDK, source-control metadata, environment files, PHP shells, SQL dumps, Java/Magento configuration, cloud/container/IDE credentials, procfs and passwd probes |

`crates/mw-server/data/scan_probe_paths.json` holds the 235 immediate catalog
entries. Additional existing filename/environment rules cover known variants.
`admin_probe_paths.json` contains the 284 editable administrative defaults and
is checked to include every immediate entry. Whole path segments are matched,
case-insensitively, including nested installations. Normalization bounds request
length and percent decoding, resolves dot segments and ignores query/fragment
text. A path substring in a search query is not scanner evidence.

## Source review

Vendor routes were checked against primary documentation; security wordlists
were used to discover aliases and deployment variations, rather than imported
wholesale. Entries identify a foreign product on Mailwoman, not a claim that the
product or every such endpoint is vulnerable.

- [WordPress hardening](https://developer.wordpress.org/advanced-administration/security/hardening/): wp-admin, wp-includes and wp-config.
- [Joomla administrator application](https://docs.joomla.org/Administrator_%28Application%29/en) and [TYPO3 login](https://docs.typo3.org/m/typo3/tutorial-editors/main/en-us/Login/Index.html): identifiable backend routes; renamed installations cannot be exhaustively enumerated.
- [phpMyAdmin installation](https://docs.phpmyadmin.net/en/latest/setup.html): setup/install surfaces and deployment variants.
- [Roundcube installation](https://github.com/roundcube/roundcubemail/wiki/Installation): installer directories for older versions and `installer.php` for v1.7+.
- [RainLoop configuration](https://www.rainloop.net/docs/configuration/) and [SnappyMail installation](https://github.com/the-djmaze/snappymail/wiki/Installation-instructions): administration and data-directory exposure. RainLoop's root `?admin` is a query selector, so the root path alone is deliberately not classified. Explicit foreign installation/data paths are covered.
- [Exchange admin center](https://learn.microsoft.com/en-us/exchange/architecture/client-access/exchange-admin-center): ECP virtual directory.
- [cPanel service URLs](https://docs.cpanel.net/knowledge-base/accounts/how-to-log-in-to-your-server-or-account/): hosting-panel routes.
- [Tomcat Manager](https://tomcat.apache.org/tomcat-11.0-doc/manager-howto.html) and [Host Manager source](https://github.com/apache/tomcat/blob/main/webapps/docs/host-manager-howto.xml): HTML and text management endpoints.
- [Apache mod_status](https://httpd.apache.org/docs/2.4/mod/mod_status.html): server-status.
- [Spring Actuator endpoints](https://docs.spring.io/spring-boot/reference/actuator/endpoints.html): environment, heap dumps, configuration, mappings, logs and other sensitive management endpoints; generic health/metrics are excluded from immediate matching.
- [SecLists common paths](https://github.com/danielmiessler/SecLists/blob/master/Discovery/Web-Content/common.txt): broader CMS, panel, database, appliance and infrastructure aliases, curated to avoid normal first-party routes.

## Boundaries and false-positive controls

Generic `/login`, `/user/login`, health, metrics, OpenAPI/Swagger, mail discovery
and ordinary static assets do not trigger immediate blocking. Mailwoman's actual
configured console route is exempt from foreign-console matching; foreign scan
suffixes beneath it remain recognizable. API/JMAP/DAV resource identifiers and
attachment filenames are excluded, even when a filename looks like `.env` or a
CMS path. Existing explicit `cmd_sco` rules remain independent of that exclusion.
Tests cover every catalog entry, nested/case/encoded requests, source exemptions,
observe/off controls, forwarding spoofing, history and source-wide enforcement.

Scored generic administration guesses retain their existing policy. Do not infer
that an arbitrary login name is a bot just because it contains `admin` or has not
yet signed in locally: proxy deployments can legitimately admit first-time
upstream users. Credential rejection and unavailable upstreams have their own
sign-in protections; this catalog changes path-based enforcement.
