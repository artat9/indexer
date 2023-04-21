import path from 'path';
import * as indexer from '../../declarations/indexer';
import { Event } from '../../declarations/indexer/indexer.did';
import fetch from 'node-fetch';
import { JsonRpcProvider, Interface } from 'ethers';
import { Erc20__factory } from '../types/ethers-contracts/factories/Erc20__factory';
import { ActorSubclass } from '@dfinity/agent';
const nodeFetch: any = fetch;
global.Headers = nodeFetch.Headers;
const WETH_ADDRESS = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
const canister = () => {
  const localCanisters = require(path.resolve(
    __dirname,
    '../../../.dfx/local/canister_ids.json'
  ));
  return indexer.createActor(localCanisters.indexer.local as string, {
    agentOptions: {
      fetch: require('node-fetch'),
      host: 'http://127.0.0.1:8000',
    },
  });
};

const saveEvents = async (events: Event[]) => {
  const map = events.reduce((map, obj) => {
    const key = obj.block_number;
    if (!map.has(key)) {
      map.set(key, []);
    }
    map.get(key)!.push(obj);
    return map;
  }, new Map<bigint, Event[]>());
  for (const [k, v] of map) {
    await canister().update_events(k, v);
  }
};

const events_from_to = async (from: number, to: number): Promise<Event[]> => {
  const contract = erc20Contract();
  const logs = await provider().getLogs({
    address: WETH_ADDRESS,
    toBlock: to,
    fromBlock: from,
    topics: [contract.interface.getEvent('Transfer').topicHash],
  });
  const erc20ContractIface = new Interface(Erc20__factory.abi);

  return logs
    .map((l) => {
      return {
        log: erc20ContractIface.parseLog({
          data: l.data,
          topics: l.topics.map((t) => t.toString()),
        })!,
        metadata: {
          hash: l.blockHash,
          blockNumber: l.blockNumber,
          at: l.blockNumber,
        },
      };
    })
    .map((t) => {
      return {
        block_number: BigInt(t.metadata.blockNumber),
        from: t.log.args[0],
        hash: t.metadata.hash,
        to: t.log.args[1],
        value: BigInt(t.log.args[2]),
        recipient: t.log.args[1],
        at: BigInt(t.metadata.at),
      };
    });
};

const provider = () => {
  return new JsonRpcProvider('https://mainnet.infura.io/v3/TEST');
};
const erc20Contract = (address?: string) => {
  return Erc20__factory.connect(address || WETH_ADDRESS, provider());
};

const main = async () => {
  const instance = canister();
  const bn_at_deploy = await instance.block_number_at_deploy();
  console.log(bn_at_deploy);
  const events = await events_from_to(17078925, 17078925 + 100);
  await saveEvents(events);
};

main();
