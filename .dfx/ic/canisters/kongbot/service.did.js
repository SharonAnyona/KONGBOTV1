export const idlFactory = ({ IDL }) => {
  const InitOrUpgradeArgs = IDL.Record({ 'oc_public_key' : IDL.Text });
  return IDL.Service({});
};
export const init = ({ IDL }) => {
  const InitOrUpgradeArgs = IDL.Record({ 'oc_public_key' : IDL.Text });
  return [InitOrUpgradeArgs];
};
