<p align="center">
	<img src='./.github/logo.svg' width='200px' />
</p>

<h1 align="center">belugasnooze pi client</h1>

<p align="center">
	The pi client that executes instructions from <a href='https://www.belugasnooze.com' target='_blank' rel='noopener noreferrer'>https://www.belugasnooze.com</a>
</p>
<p align="center">
	Built in <a href='https://www.typescriptlang.org/' target='_blank' rel='noopener noreferrer'>Typescript</a>, for <a href='https://nodejs.org/en/' target='_blank' rel='noopener noreferrer'>Node.js</a>, with <a href='https://www.sqlite.org/' target='_blank' rel='noopener noreferrer'>SQLite</a>
</p>


## Required software

1) <a href='https://www.docker.com/' target='_blank' rel='noopener noreferrer'>Docker</a> - container runner


Suggested locations for directories required by belugasnooze

| directory | reason|
| --- | --- |
|```~/belugasnooze/```	| Location of the node app |
|```/ramdrive```		| tmpfs ramdrive, via /etc/fstab - ```tmpfs /ramdrive tmpfs defaults,noatime,size=128K     0       0```|

Files that are required by belugasnooze
| file | reason|
|---|---|
|```./.env```				| enviromental variables, make sure in production mode|
|```/ramdrive/ip.addr```	| ip address for private network, use crontab ```*/5 * * * * ip addr show wlan0 \| grep -Po 'inet \K[\d.]+' > /ramdrive/ip.addr``` |

## Build step
1) When on main branch compile typescript and install all node modules using build process ```node build```