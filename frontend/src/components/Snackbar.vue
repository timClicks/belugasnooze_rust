<template>
	<v-snackbar
		v-model='visible'
		timeout='-1'
	>
		<v-row justify='center' align='center' no-gutters class='ma-0 pa-0'>
			<v-col v-if='icon && !loading' cols='auto' class=''>
				<v-icon color='primary'>{{ icon }}</v-icon>
			</v-col>
			<v-col v-if='loading' cols='auto' class=''>
				<v-progress-circular
					indeterminate
					:width='3'
					:size='18'
					color='white'
				/>
			</v-col>
			<v-col cols='auto' class='mx-3 ma-0 pa-0'>
				{{ message }}
			</v-col>
		</v-row>
	</v-snackbar>

</template>

<script lang ='ts'>
import Vue from 'vue';

import { mapStores } from 'pinia';
import { snackbarModule } from '@/store';

export default Vue.extend({
	name: 'snackbar-component',
	
	computed: {
		...mapStores(snackbarModule),
		
		icon (): string {
			return this.snackbarStore.icon;
		},
		loading: {
			get (): boolean {
				return this.snackbarStore.loading;
			},
			set (b: boolean): void {
				this.snackbarStore.set_loading(b);
			}
		},
		message: {
			get (): string {
				return this.snackbarStore.message;
			},
			set (s: string): void {
				this.snackbarStore.set_message(s);
			}
		},
		snackTimeout: {
			get (): number {
				return this.snackbarStore.timeout;
			},
			set (nu: number): void {
				this.snackbarStore.set_timeout(nu);
			}
		},
		visible: {
			get (): boolean {
				return this.snackbarStore.visible;
			},
			set (b: boolean) : void {
				this.snackbarStore.set_visible(b);
			}
		},
	},

	data: () => ({
		timeout: 0
	}),

	methods: {
		closeSnackbar (): void {
			this.visible = false;
			this.snackbarStore.$reset();
			clearTimeout(this.timeout);
			this.timeout = 0;
		}
	},

	watch: {
		visible (i): void {
			if (i && this.snackTimeout) this.timeout = window.setTimeout(() => {
				this.closeSnackbar();
			}, this.snackTimeout);
		}
	}
});
</script>