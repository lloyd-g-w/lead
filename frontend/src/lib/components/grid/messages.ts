interface LeadMsg {
	msg_type: 'set' | 'get' | 'error' | 'bulk';
	cell?: CellRef;
	raw?: string;
	eval?: Literal;
	bulk_msgs?: Array<LeadMsg>;
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
