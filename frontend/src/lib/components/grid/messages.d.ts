interface LeadMsg {
	msg_type: 'set' | 'get' | 'error';
	cell?: CellRef;
	raw?: string;
	eval?: Literal;
}

interface CellRef {
	row: number;
	col: number;
}

type LiteralType = 'Number' | 'Boolean' | 'String';
type LiteralValue = number | string | boolean;

interface Literal {
	type: LiteralType;
	value: LiteralValue;
}
