import path from 'path';
import * as indexer from '../../declarations/indexer';
import { Event } from '../../declarations/indexer/indexer.did';
import fetch from 'node-fetch';
import { JsonRpcProvider, Interface } from 'ethers';
import { Erc20__factory } from '../types/ethers-contracts/factories/Erc20__factory';
const nodeFetch: any = fetch;
global.Headers = nodeFetch.Headers;
const WETH_ADDRESS = '0x6B175474E89094C44Da98b954EedeAC495271d0F';
const GET_LOGS_BATCH_SISE = 3000;
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
  if (events.length == 0) {
    console.log('no events found');
    return;
  }
  const map = events.reduce((map, obj) => {
    const key = obj.block_number;
    if (!map.has(key)) {
      map.set(key, []);
    }
    map.get(key)!.push(obj);
    return map;
  }, new Map<bigint, Event[]>());
  let txCount = 0;
  Array.from(map.values()).forEach((e) => (txCount += e.length));
  console.log('saving events', txCount);
  await canister().update_events([...map]);
};

const events_from_to = async (from: number, to: number): Promise<Event[]> => {
  const contract = erc20Contract();
  const getLogsFunc = async (from: number, to: number) => {
    return await provider().getLogs({
      address: WETH_ADDRESS,
      toBlock: to,
      fromBlock: from,
      topics: [contract.interface.getEvent('Transfer').topicHash],
    });
  };
  let logs = [];
  let startFrom = from;
  while (true) {
    let batchTo = startFrom + 1000 > to ? to : startFrom + 1000;
    console.log('getting logs fromTo', startFrom, batchTo);
    const batchLogs = await getLogsFunc(startFrom, batchTo);
    logs.push(...batchLogs);
    if (batchTo == to) {
      break;
    }
    startFrom = batchTo + 1;
  }
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
  const startFrom = Number(await instance.latest_block_number());
  let processed = startFrom;
  while (true) {
    let to =
      processed + GET_LOGS_BATCH_SISE > bn_at_deploy
        ? Number(bn_at_deploy)
        : processed + GET_LOGS_BATCH_SISE;
    console.log('processing sync event fromTo', processed, to);
    const events = await events_from_to(processed, to);
    await saveEvents(events);
    processed = to;
    if (processed >= bn_at_deploy) {
      break;
    }
  }
};

main();
