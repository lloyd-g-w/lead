let vimHistory = new Array<VimCommand>();

let vimState: VimState = {
	modifier: undefined,
	action: undefined
};

// Id to listener map
let vimCommandListeners: Record<string, VimListener> = {};

export interface VimListener {
	onVimCommand(command: VimCommand): void;
}

export type VimModifier = number;

export type VimMotion = 'up' | 'down' | 'left' | 'right';

export type VimAction = 'delete' | 'insert-mode' | 'visual-mode' | 'escape' | 'goto';

export interface VimState {
	modifier?: VimModifier;
	action?: VimAction;
}

export interface VimCommand {
	action?: VimAction;
	modifier?: VimModifier;
	motion?: VimMotion;
}

function emitVimCommand(command: VimCommand): void {
	for (let key in vimCommandListeners) {
		vimCommandListeners[key].onVimCommand(command);
	}
}

function resetVimState(): void {
	vimState = {
		modifier: undefined,
		action: undefined
	};
}

export function registerVimListener(id: string, listener: VimListener): void {
	if (id in vimCommandListeners) {
		return;
	}
	vimCommandListeners[id] = listener;
}

export function unregisterVimListener(id: string): void {
	delete vimCommandListeners[id];
}

export function getVimId(): string {
	let id = 'vim-listener-' + Math.random().toString(36).slice(2, 11);

	while (id in vimCommandListeners) {
		id = 'vim-listener-' + Math.random().toString(36).slice(2, 11);
	}

	return id;
}

export function vimKeyboardHandler(e: KeyboardEvent): void {
	const motionMap: Record<string, VimMotion> = {
		h: 'left',
		j: 'down',
		k: 'up',
		l: 'right'
	};

	switch (e.key) {
		case 'i':
			emitVimCommand({ action: 'insert-mode' });
			resetVimState();
			e.preventDefault();
			break;

		case 'v':
			emitVimCommand({ action: 'visual-mode' });
			resetVimState();
			e.preventDefault();
			break;

		case 'g':
			vimState.action = 'goto';
			e.preventDefault();
			break;

		case 'h':
		case 'j':
		case 'k':
		case 'l':
			emitVimCommand({
				modifier: vimState.modifier,
				action: vimState.action,
				motion: motionMap[e.key]
			});
			resetVimState();
			e.preventDefault();
			break;

		case '.':
			if (vimHistory.length > 0) {
				const lastCommand = vimHistory[vimHistory.length - 1];
				emitVimCommand(lastCommand);
				resetVimState();
				e.preventDefault();
			}
			break;

		case '0':
		case '1':
		case '2':
		case '3':
		case '4':
		case '5':
		case '6':
		case '7':
		case '8':
		case '9':
			if (vimState.modifier === undefined) {
				if (e.key === '0') {
					// Leading zero, ignore
					e.preventDefault();
					break;
				}
				vimState.modifier = parseInt(e.key) || 1;
			} else {
				vimState.modifier = vimState.modifier * 10 + (parseInt(e.key) || 0);
			}
			e.preventDefault();
			break;

		case 'Escape':
			emitVimCommand({ action: 'escape' });
			resetVimState();
			e.preventDefault();

		default:
			return;
	}
}
