<script lang="ts">
	import { useMutation } from '@urql/vue';

	import { defineAsyncComponent, reactive } from 'vue';

	import { useInstanceInfo } from '../graphql/instance-info';
	import { graphql } from '../graphql/types';
	import { authorizationUrl } from '../lib/oauth2';
	import BaseModal from './modal/BaseModal.vue';

	const CaptchaComponent = defineAsyncComponent(() => import('./CaptchaComponent.vue'));

	const modalData = reactive({
		show: false,
		title: '',
		content: ''
	});

	type RegisterData = {
		username: string;
		email: string;
		password: string;
		captchaToken: string;
	};

	const registerUser = useMutation(
		graphql(`
			mutation registerUser(
				$username: String!
				$email: String!
				$password: String!
				$captchaToken: String
			) {
				registerUser(
					username: $username
					email: $email
					password: $password
					captchaToken: $captchaToken
				) {
					id
				}
			}
		`)
	);

	function registerDone() {
		modalData.title = 'Register';
		modalData.content = 'Registered successful!';
		modalData.show = true;
	}

	function registerError(err: Error) {
		modalData.title = 'Register';
		modalData.content = `Registration failed: ${err.message}`.replaceAll('\n', '<br />');
		modalData.show = true;
	}

	const instanceData = useInstanceInfo();

	async function login(): Promise<void> {
		const url = await authorizationUrl();
		window.location.href = url;
	}

	async function register(registerData: RegisterData): Promise<void> {
		try {
			const result = await registerUser.executeMutation({
				username: registerData.username,
				email: registerData.email,
				password: registerData.password,
				captchaToken: registerData.captchaToken
			});

			if (result.error) {
				registerError(result.error);
			} else {
				registerDone();
			}
		} catch (error) {
			registerError(error as Error);
		}
	}
</script>

<div class="forms">
	<FormKit type="form" name="login-form" submit-label="Login" @submit="login" />

	<FormKit
		v-if="instanceData?.registrationsOpen"
		type="form"
		submit-label="Register"
		@submit="register"
	>
		<FormKit
			type="text"
			name="username"
			validation="required"
			label="Username"
			placeholder="aumetra"
		/>

		<FormKit
			type="email"
			name="email"
			validation="email|required"
			label="Email address"
			placeholder="aumetra@citadel-station.example"
		/>

		<FormKit
			type="password"
			name="password"
			validation="required|zxcvbn"
			validation-visibility="dirty"
			label="Password"
		/>

		<FormKit
			type="password"
			name="password_confirm"
			validation="required|confirm"
			label="Confirm password"
			validation-label="Password confirmation"
		/>

		<CaptchaComponent
			v-if="instanceData?.captcha"
			:backend="instanceData?.captcha?.backend"
			:sitekey="instanceData?.captcha?.key"
		/>
	</FormKit>

	<BaseModal v-model="modalData.show" title="modalData.title">
		<!-- This is returned from the backend and created from an error type, and only "enhanced" with HTML newlines by us -->
		{@html modalData.content}

		<p>
			<button onclick={() => (modalData.show = false)}>Close</button>
		</p>
	</BaseModal>
</div>

<style lang="scss">
	@use '../../styles/colours' as *;

	form[name='login-form'] {
		align-items: center;
	}

	.formkit-form {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		margin: 0 auto;
		border: 0.2px solid $shade1dark;
		border-radius: 5px;
		background-color: $dark2;
		padding: 3vh;
		width: 90%;
	}

	.formkit-wrapper {
		margin: 10px auto;
	}

	.formkit-input[type='submit'] {
		transition: 0.5s;
		cursor: pointer;
		border: 0;
		border-radius: 5px;
		background-color: $shade1dark;
		padding: 10px;
		width: 100px;
		font-size: 16px;

		&:hover {
			background-color: $shade2dark;
		}
	}

	.formkit-input:not([type='submit']) {
		border: 0.5px solid $shade1dark;
		border-radius: 2px;
		background-color: $dark1;
		padding: 5px;
		width: 100%;
		color: white;
		font-size: 20px;
	}

	.forms {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: flex-end;
		gap: 20px;
		padding: 1vw;
		width: 40%;

		@media only screen and (max-width: 1367px) {
			align-items: center;
			width: 45%;
		}

		@media only screen and (max-width: 1023px) {
			width: 66%;
		}
	}

	.formkit-messages {
		padding-left: 0;
		color: red;
		list-style: none;
	}

	.formkit-label {
		margin-bottom: 5px;
		text-transform: uppercase;
	}
</style>
