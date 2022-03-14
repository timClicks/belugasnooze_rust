<template>
	<v-footer
		absolute
		padless
		color='transparent'
	>
		<v-row @click='buildInfo' justify='center' align='center' class='py-0 ma-0 no-gutters'>
			<v-col cols='auto' class='py-0 ma-0 no-gutters'>
				<span class='text-caption font-weight-black'><span v-if='showBuild'>site: </span>{{ appVersion }}</span>
				<span v-if='showBuild' class='text-caption font-weight-black ml-3'>build: {{ buildDate }}</span>
				<span v-if='showBuild' class='text-caption font-weight-black ml-3'>backend_version: {{ api_version }}</span>
			</v-col>
		</v-row>
			
	</v-footer>
</template>

<script lang='ts'>

import Vue from 'vue';

import { mapStores } from 'pinia';
import { userModule, piStatusModule } from '@/store';
import { env } from '@/vanillaTS/env';

export default Vue.extend({
	name: 'footer-component',

	async beforeDestroy () {
		clearTimeout(this.buildTimeout);
	},

	computed: {
		...mapStores(userModule),
		
		api_version (): string {
			return piStatusModule().piVersion;
		},
		authenticated (): boolean {
			return this.userStore.authenticated;
		},
	},

	data: () => ({
		appVersion: env.app_version,
		buildDate: env.build_date,
		buildTimeout: 0,
		showBuild: false,
	}),

	methods: {
		/**
		 ** Show build date on version number click if authed
		 */
		buildInfo (): void {
			if (!this.authenticated || this.showBuild) return;
			this.showBuild = !this.showBuild;
			clearTimeout(this.buildTimeout);
			this.buildTimeout = window.setTimeout(() => {
				this.showBuild = false;
			}, 10000);
		},
	},

	watch: {
		/**
		 ** Watch authenticated, remove showbuild if signouted
		 */
		authenticated (i: boolean): void {
			if (!i) this.showBuild = false;
		}
	},
});
</script>