
// import { exec as unpromise_exec } from 'child_process';
// import { promisify } from 'util';
import { alarmController } from './alarm';
import { api_version as piVersion } from '../config/api_version';
import { isTimeZone } from '../types/typeguards';
import { LOCATION_IP_ADDRESS } from '../config/env';
import { promises as fs } from 'fs';
import { queries } from './queries';
import { TWifi, TPiStatus } from '../types';
import { uptime } from 'os';

export const getIp = async (): Promise<string> => {
	const ip_address = await fs.readFile(LOCATION_IP_ADDRESS, 'utf-8');
	return ip_address.trim();
};
// const exec = promisify(unpromise_exec);

/**
 ** Add a new wifi network, amend wpa_supplicant.conf
 *
 * @param ssid -  REQUIRED ssid name
 * @param password -  ssid password - optional
 */
export const addWifi = async ({ ssid, password }: TWifi): Promise<void> => {
	if (!ssid) return;
	console.log(`todo: wifi: ${ssid}:${password}`);
	// const typedLines = [
	// 	'',
	// 	'network={',
	// 	`        ssid="${ssid}"`,
	// 	`        psk="${password}"`,
	// 	`}`
	// ];
	// TODO make a backup of the config, was can restore
	// TODO this needs to be better in general, shouldn't rely on sudo commands
	// can reschedule alarms with timesonze in node cron, and then adjust piTime with a tz offset
	// eslint-disable-next-line no-await-in-loop
	// for (const line of typedLines) await exec(`echo '${line}' | sudo tee -a /etc/wpa_supplicant/wpa_supplicant.conf`);
};

export const piStatus = async (): Promise<TPiStatus> => {

	const timeZone = queries.select_timezone();

	const tzOptions: Intl.DateTimeFormatOptions = {
		timeZone,
		hour: 'numeric',
		minute: 'numeric',
		second: 'numeric',
		hour12: false
	};
	const formattedDate = new Intl.DateTimeFormat([], tzOptions);
	
	// xx:xx:xx-region/zone
	const piTime = `${formattedDate.format(new Date())}-${timeZone}`;

	const output: TPiStatus = {
		internalIp: await getIp(),
		piVersion,
		piTime,
		piNodeUptime: Math.trunc(process.uptime()),
		piUptime: Math.trunc(uptime())
	};
	return output;
};

export const quit = (): void => {
	process.exit();
};

export const setTimeZone = (data: string) : void => {
	if (!isTimeZone(data)) return;
	queries.update_timezone(data);
	alarmController.selectAndSchedule();
};