// will use this file for declarin interfaces to well maintain code
export interface transfer {
  trxHash: string;
  timestamp: string;
  from: string;
  to: string;
  amount: any;
  token: string;
}

export interface token {
  token: string;
  blockNumber: string;
  transactionHash: string;
  ownerAddress: string;
  timestamp: string;
  isGraduated: string;
}
