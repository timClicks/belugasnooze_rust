import { defineStore } from 'pinia';
import { ModuleName } from '@/types/enum_module';
import { TTime } from '@/types';

export const piStatusModule = defineStore(ModuleName.Pistatus, {

	state: () => ({
		init: false,
		internalIp: '',
		online: false,
		connectedFor: 0,
		piNodeUptime: 0,
		piUptime: 0,
		piVersion: '',
		serverNodeUptime: 0,
		serverUptime: 0,
		time: { hours: 0, minutes: 0, seconds: 0 } as TTime,
		timeZone: '',
	}),

	actions: {
		set_connectedFor (n: number): void {
			this.connectedFor = n;
		},
		set_init (b: boolean): void {
			this.init = b;
		},
		set_internalIp (su: string): void {
			this.internalIp = su;
		},
		set_online (b: boolean): void {
			this.online = b;
			this.init = true;
		},
		set_piNodeUptime (nu: number): void {
			this.piNodeUptime = nu;
		},
		set_piUptime (nu: number): void {
			this.piUptime = nu;
		},
		set_piVersion (su: string): void {
			this.piVersion = su;
		},
		set_serverNodeUptime (nu: number): void {
			this.serverNodeUptime = nu;
		},
		set_serverUptime (nu: number): void {
			this.serverUptime = nu;
		},
		set_time (t: TTime): void {
			this.time = t;
		},
		set_timeZone (t: string): void {
			this.timeZone = t;
		}
	}
});
