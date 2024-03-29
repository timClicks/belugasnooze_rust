import { snackError, snackReset } from './snack';
import { userModule, wsModule, } from '@/store';
import Axios, { AxiosInstance, AxiosError } from 'axios';
import { env } from '@/vanillaTS/env';

const wrap = <T> () => {
	return function (_target: AxiosRequests, _propertyKey: string, descriptor: PropertyDescriptor): void {
		const original = descriptor.value;
		descriptor.value = async function (t: T): Promise<void> {
			try {
				const result = await original.call(this, t);
				return result;
			} catch (err) {
				const e = <AxiosError>err;
				if (e.message === 'offline') snackError({ message: 'Server offline' });
				else if (e.response?.status === 403) {
					const [ user_store, ws_store ] = [ userModule(), wsModule() ];
					if (user_store.authenticated) user_store.signout();
					ws_store.closeWS();
					snackError({ message: 'invalid password' });
				} else if (e.response?.status === 429) {
					const converted = Math.ceil(e.response.data.response / 1000);
					const message = `too many requests - please try again in ${converted} seconds `;
					snackError({ message });
				} else {
					snackError({ message: e?.response?.data?.response });
				}
			}
		};
	};
};

class AxiosRequests {

	#wsAuthAxios!: AxiosInstance

	constructor () {
		this.#wsAuthAxios = Axios.create({
			baseURL: env.domain_auth,
			withCredentials: true,
			headers: {
				'Accept': 'application/json',
				'Content-Type': 'application/json; charset=utf-8',
				'Cache-control': 'no-cache'
			},
		});
	
		this.#wsAuthAxios.interceptors.response.use(
			(config) => Promise.resolve(config),
			(error) => error.response ? Promise.reject(error) : Promise.reject(new Error('offline'))
		);
	}

	@wrap()
	async wsAuth_post (password: string): Promise<boolean> {
		const { data } = await this.#wsAuthAxios.post('/', { key: env.api_key, password });
		if (data.response) {
			snackReset();
			userModule().set_authenticated(true);
			wsModule().openWS(data?.response);
		}
		return !!data.response;
	}
}

export const axiosRequests = new AxiosRequests();