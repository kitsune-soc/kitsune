import { setClientSession, type ClientPlugin } from '$houdini';

const houdiniPlugin: ClientPlugin = () => {
	return {
		async start(ctx, { next }) {
			setClientSession({ headers: { owo: 'uwu' } });
			next(ctx);
		}
	};
};

export { houdiniPlugin };
