use std::time::Duration;

use alloy_eips::eip1559::BaseFeeParams;
use alloy_network::{Ethereum, EthereumWallet, NetworkWallet, TransactionBuilder, TxSigner};
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::{Address, B256, Bytes, hex, TxKind};
use alloy_provider::{Network, Provider};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionInput, TransactionRequest};
use alloy_rpc_types_trace::geth::AccountState;
use alloy_signer_local::LocalWallet;
use alloy_signer_local::PrivateKeySigner;
use alloy_transport::Transport;
use eyre::{eyre, OptionExt, Result};
use k256::{Secp256k1, SecretKey};
use lazy_static::lazy_static;
use log::{debug, error};

use debug_provider::AnvilProviderExt;

lazy_static! {
    static ref NO_OWNER_CODE : Vec<u8> = hex::decode("3615612c16575f3560e01c806328472417146115b2578063fa461e33146115e357806323a69e75146115e35780638b418713146115aa57806320c13b0b14612ae75780631626ba7e14612af7578063f04f2707146100e9578063ab6291fe146115ba578063923b8a2a146115c25750606435806080146100b75750604435806060146115e35750600435806020146100af5750602435806040146100af57506084358060a0146100af5750612c16565b600401611665565b8035601414612b07575060243580156100d2576020526100da565b506044356020525b60205f52606435600401611665565b5060643560040180358091602001610120375961010052610120016101205b8080518060f01c806180001615156101fd57828260b01c62ffffff1680156101b9578062ffffff146101b9579081610fff160181600c1c60ff1680156101b757908260141c60071660051b6020018362800000161561016957602090035f51035b905b8215156101795750506101b1565b826020116101985781518152602001906020019160209003919061016b565b905160018360031b1b6001900380199116908251161790525b506101bb565b505b505b5061ffff1680156115575780617ffc146102b55780617ffe14610dc65780617ffb146104325780617fff14610e5f5780617ffd14610ef85780617ffa1461035e575b508060b01c697fffffffffffffffffff1691908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a579491505015156102ab578060c81c62ffffff1680156102a9578062ffffff146102a95780610fff1681600c1c60ff16908260141c60071660051b836280000016156102a4575f510380830160051c60051b83601f16156102a1576020015f81525b5f525b6020013e5b505b505060200161155d565b505f91908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a57949150501515610354578060c81c62ffffff168015610352578062ffffff146103525780610fff1681600c1c60ff16908260141c60071660051b8362800000161561034d575f510380830160051c60051b83601f161561034a576020015f81525b5f525b6020013e5b505b505060200161155d565b508060b01c62ffffff168060141c60071660051b6020018162800000161561038857602090035f51035b51905091908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a57949150501515610428578060c81c62ffffff168015610426578062ffffff146104265780610fff1681600c1c60ff16908260141c60071660051b83628000001615610421575f510380830160051c60051b83601f161561041e576020015f81525b5f525b6020013e5b505b505060200161155d565b5060a01c61ffff16600c0190600c015b805160f81c8015610db5576010811161056157806001146104cb57806002146104d657806003146104e157806004146104ec57806005146104f75780600614610502578060071461050d57806008146105185780600a1461052b5780600b146105345780600c1461053d5780600d146105465780600e1461054f5780600f146105585750610db5565b506020516001610dad565b506040516001610dad565b506060516001610dad565b506080516001610dad565b5060a0516001610dad565b5060c0516001610dad565b5060e0516001610dad565b505f51805190602090035f526001610dad565b50476001610dad565b50416001610dad565b50426001610dad565b50436001610dad565b50486001610dad565b503a6001610dad565b60208110156106a057806011146105e857806012146105f357806013146105fe57806014146106095780601514610614578060161461061f578060171461062a578060181461063557806019146106405780601a1461064a5780601b1461065e5780601c146106725780601d1461067f5780601e1461068c5780601f146106965750610db5565b509190016001610dad565b509190036001610dad565b509190026001610dad565b509190046001610dad565b509190056001610dad565b509190166001610dad565b509190176001610dad565b509190186001610dad565b5090196001610dad565b50806001015160f81c9091901b6002610dad565b50806001015160f81c9091901c6002610dad565b5090600190016001610dad565b5090600190036001610dad565b5091036001610dad565b5091046001610dad565b60308110156107d657806020146106ef578060211461070057806022146107105780602314610734578060241461074b578060251461076857806026146107875780602a146107aa5750610db5565b5080600101516021610dad56610db5565b50806001015160ff166002610dad565b50806001015161ffff166003610dad565b50806001015163ffffffff166005610dad565b50806001015167ffffffffffffffff166009610dad565b5080600101516dffffffffffffffffffffffffffff166007610dad565b5080600101516fffffffffffffffffffffffffffffffff166009610dad565b50806001015173ffffffffffffffffffffffffffffffffffffffff16600d610dad565b50907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff90026001610dad565b6030811015610894578060311461080d578060321461081857806033146108325780603414610844578060351461085c5750610db5565b5060ff166001610dad565b5061ffff166001610dad565b5063ffffffff166001610dad565b5067ffffffffffffffff166001610dad565b506dffffffffffffffffffffffffffff166001610dad565b506fffffffffffffffffffffffffffffffff166001610dad565b5073ffffffffffffffffffffffffffffffffffffffff166001610dad565b6050811015610d275780604014610923578060411461098b57806042146109f25780604314610a5a5780604414610ac15780604514610b295780604614610b905780604714610bf55780604814610c5b5780604914610c755780604a14610c8e5780604b14610ca85780604c14610cc15780604d14610cdb5780604e14610cf45780604f14610d0b5750610db5565b50919014156109355760016001610dad565b7f455100000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5091901461099c5760016001610dad565b7f4e4551000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5091901015610a045760016001610dad565b7f4c5400000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919010610a6b5760016001610dad565b7f475445000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5091901115610ad35760016001610dad565b7f475400000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919011610b3a5760016001610dad565b7f4c5445000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5090610b9f5760016001610dad565b7f5a5200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b509015610c055760016001610dad565b7f4e5a52000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5091901415610c6d5760016001610dad565b5f5f5260205ff35b50919014610c865760016001610dad565b5f5f5260205ff35b5091901015610ca05760016001610dad565b5f5f5260205ff35b50919010610cb95760016001610dad565b5f5f5260205ff35b5091901115610cd35760016001610dad565b5f5f5260205ff35b50919011610cec5760016001610dad565b5f5f5260205ff35b5090610d035760016001610dad565b5f5f5260205ff35b509015610d1b5760016001610dad565b5f5f5260205ff3610db5565b6060811015610da75780605014610d465780605114610d7a5750610db5565b50806001015160f81c8060051b5f5103805f525f5b9060200190815101916001900391821515610d5b579150506002610dad565b50806001015160f81c5f515f5b815101906020900390916001900391821515610d87579150506002610dad565b50610db5565b909101610442565b50505f516020019081525f5261155d565b508073ffffffffffffffffffffffffffffffffffffffff168160a01c61ffff165f5f8260208701855afa15612c1a5792505060c81c62ffffff168015610e56578062ffffff14610e565780610fff1681600c1c60ff16908260141c60071660051b83628000001615610e51575f510380830160051c60051b83601f1615610e4e576020015f81525b5f525b6020013e5b5060200161155d565b508073ffffffffffffffffffffffffffffffffffffffff168160a01c61ffff165f5f8260208701855af415612c1a5792505060c81c62ffffff168015610eef578062ffffff14610eef5780610fff1681600c1c60ff16908260141c60071660051b83628000001615610eea575f510380830160051c60051b83601f1615610ee7576020015f81525b5f525b6020013e5b5060200161155d565b508060a01c61ffff1682600c015160e01c8063a85f1d24146113895780634df86adf146112a457806305ec9cad146114445780639b81788b14610f8c5780638bceaa1814610fef57806384f16ca01461105157806395b66162146110b45780639a23842e146111165780634fae2f231461117a578063a9f2812f146111dd578063f93a171614611241576199995f5260205ffd5b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa15610fc957905080602001519051610fce565b50505f5f5b9082026127100291900390910290046001015f516020019081525f5261154c565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa1561102c57905080602001519051611031565b50505f5f5b82026127100291900390910290046001015f516020019081525f5261154c565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa1561108e57905080602001519051611093565b50505f5f5b9090919092029182029190612710020190045f516020019081525f5261154c565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa156110f1579050806020015190516110f6565b50505f5f5b90919092029182029190612710020190045f516020019081525f5261154c565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa1561115457905080602001519051611159565b50505f5f5b9082026127100291900390910290046001015f516020019081525f5261154c565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa156111b8579050806020015190516111bd565b50505f5f5b82026127100291900390910290046001015f516020019081525f5261154c565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa1561121b57905080602001519051611220565b50505f5f5b9090919092029182029190612710020190045f516020019081525f5261154c565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa1561127f57905080602001519051611284565b50505f5f5b90919092029182029190612710020190045f516020019081525f5261154c565b5083600c0180600401518160240151826044015182610100516370a082318152308160200152602081602483601c01855afa15612c1a575180156112e757600190035b80841115611345577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b85606401516101005163a9059cbb81529081602001529081604001525f5f604483601c015f865af15050508015611380575f5f5f5f84415af1505b5050505061154c565b5083600c0180600401518160240151826044015182610100516370a082318152308160200152602081602483601c01855afa15612c1a575180156113cc57600190035b8084111561142a577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5050801561143c575f5f5f5f84415af1505b50505061154c565b5083600c0173c02aaa39b223fe8d0a0e5c4f27ead9083c756cc28160040151826024015182610100516370a082318152308160200152602081602483601c01855afa15612c1a5751801561149757600190035b808411156114f5577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b61010051632e1a7d4d81529081602001525f5f602483601c015f73c02aaa39b223fe8d0a0e5c4f27ead9083c756cc25af150508115611538575f5f5f5f85415af1505b84604401515f5f5f475f945af15050505050505b915050600c0161155d565b5061ffff015b01818110610108575b505060043560240135602435602401356044356024013501336101005163a9059cbb81529081602001529081604001525f5f604483601c015f865af1505050612c22565b506084611665565b506024611665565b506004611665565b6044356004018035601414612bcf575060043560405260243560205261165a565b506044356004018035601414612bcf57506004355f811261162e576040526024357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0260205261165a565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff026020526024356040525b60405f526044356004015b80358091602001610120375961010052610120016101205b8080518060f01c8061800016151561177257828260b01c62ffffff16801561172e578062ffffff1461172e579081610fff160181600c1c60ff16801561172c57908260141c60071660051b602001836280000016156116de57602090035f51035b905b8215156116ee575050611726565b8260201161170d578151815260200190602001916020900391906116e0565b905160018360031b1b6001900380199116908251161790525b50611730565b505b505b5061ffff168015612acc5780617ffc1461182a5780617ffe1461233b5780617ffb146119a75780617fff146123d45780617ffd1461246d5780617ffa146118d3575b508060b01c697fffffffffffffffffff1691908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a57949150501515611820578060c81c62ffffff16801561181e578062ffffff1461181e5780610fff1681600c1c60ff16908260141c60071660051b83628000001615611819575f510380830160051c60051b83601f1615611816576020015f81525b5f525b6020013e5b505b5050602001612ad2565b505f91908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a579491505015156118c9578060c81c62ffffff1680156118c7578062ffffff146118c75780610fff1681600c1c60ff16908260141c60071660051b836280000016156118c2575f510380830160051c60051b83601f16156118bf576020015f81525b5f525b6020013e5b505b5050602001612ad2565b508060b01c62ffffff168060141c60071660051b602001816280000016156118fd57602090035f51035b51905091908073ffffffffffffffffffffffffffffffffffffffff16818060a01c61ffff165f5f826020880189875af115612c1a5794915050151561199d578060c81c62ffffff16801561199b578062ffffff1461199b5780610fff1681600c1c60ff16908260141c60071660051b83628000001615611996575f510380830160051c60051b83601f1615611993576020015f81525b5f525b6020013e5b505b5050602001612ad2565b5060a01c61ffff16600c0190600c015b805160f81c801561232a5760108111611ad65780600114611a405780600214611a4b5780600314611a565780600414611a615780600514611a6c5780600614611a775780600714611a825780600814611a8d5780600a14611aa05780600b14611aa95780600c14611ab25780600d14611abb5780600e14611ac45780600f14611acd575061232a565b506020516001612322565b506040516001612322565b506060516001612322565b506080516001612322565b5060a0516001612322565b5060c0516001612322565b5060e0516001612322565b505f51805190602090035f526001612322565b50476001612322565b50416001612322565b50426001612322565b50436001612322565b50486001612322565b503a6001612322565b6020811015611c155780601114611b5d5780601214611b685780601314611b735780601414611b7e5780601514611b895780601614611b945780601714611b9f5780601814611baa5780601914611bb55780601a14611bbf5780601b14611bd35780601c14611be75780601d14611bf45780601e14611c015780601f14611c0b575061232a565b509190016001612322565b509190036001612322565b509190026001612322565b509190046001612322565b509190056001612322565b509190166001612322565b509190176001612322565b509190186001612322565b5090196001612322565b50806001015160f81c9091901b6002612322565b50806001015160f81c9091901c6002612322565b5090600190016001612322565b5090600190036001612322565b5091036001612322565b5091046001612322565b6030811015611d4b5780602014611c645780602114611c755780602214611c855780602314611ca95780602414611cc05780602514611cdd5780602614611cfc5780602a14611d1f575061232a565b50806001015160216123225661232a565b50806001015160ff166002612322565b50806001015161ffff166003612322565b50806001015163ffffffff166005612322565b50806001015167ffffffffffffffff166009612322565b5080600101516dffffffffffffffffffffffffffff166007612322565b5080600101516fffffffffffffffffffffffffffffffff166009612322565b50806001015173ffffffffffffffffffffffffffffffffffffffff16600d612322565b50907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff90026001612322565b6030811015611e095780603114611d825780603214611d8d5780603314611da75780603414611db95780603514611dd1575061232a565b5060ff166001612322565b5061ffff166001612322565b5063ffffffff166001612322565b5067ffffffffffffffff166001612322565b506dffffffffffffffffffffffffffff166001612322565b506fffffffffffffffffffffffffffffffff166001612322565b5073ffffffffffffffffffffffffffffffffffffffff166001612322565b605081101561229c5780604014611e985780604114611f005780604214611f675780604314611fcf5780604414612036578060451461209e5780604614612105578060471461216a57806048146121d057806049146121ea5780604a146122035780604b1461221d5780604c146122365780604d146122505780604e146122695780604f14612280575061232a565b5091901415611eaa5760016001612322565b7f455100000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919014611f115760016001612322565b7f4e4551000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b5091901015611f795760016001612322565b7f4c5400000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919010611fe05760016001612322565b7f475445000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919011156120485760016001612322565b7f475400000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b509190116120af5760016001612322565b7f4c5445000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50906121145760016001612322565b7f5a5200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50901561217a5760016001612322565b7f4e5a52000000000000000000000000000000000000000000000000000000000060037f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b50919014156121e25760016001612322565b5f5f5260205ff35b509190146121fb5760016001612322565b5f5f5260205ff35b50919010156122155760016001612322565b5f5f5260205ff35b5091901061222e5760016001612322565b5f5f5260205ff35b50919011156122485760016001612322565b5f5f5260205ff35b509190116122615760016001612322565b5f5f5260205ff35b50906122785760016001612322565b5f5f5260205ff35b5090156122905760016001612322565b5f5f5260205ff361232a565b606081101561231c57806050146122bb57806051146122ef575061232a565b50806001015160f81c8060051b5f5103805f525f5b90602001908151019160019003918215156122d0579150506002612322565b50806001015160f81c5f515f5b8151019060209003909160019003918215156122fc579150506002612322565b5061232a565b9091016119b7565b50505f516020019081525f52612ad2565b508073ffffffffffffffffffffffffffffffffffffffff168160a01c61ffff165f5f8260208701855afa15612c1a5792505060c81c62ffffff1680156123cb578062ffffff146123cb5780610fff1681600c1c60ff16908260141c60071660051b836280000016156123c6575f510380830160051c60051b83601f16156123c3576020015f81525b5f525b6020013e5b50602001612ad2565b508073ffffffffffffffffffffffffffffffffffffffff168160a01c61ffff165f5f8260208701855af415612c1a5792505060c81c62ffffff168015612464578062ffffff146124645780610fff1681600c1c60ff16908260141c60071660051b8362800000161561245f575f510380830160051c60051b83601f161561245c576020015f81525b5f525b6020013e5b50602001612ad2565b508060a01c61ffff1682600c015160e01c8063a85f1d24146128fe5780634df86adf1461281957806305ec9cad146129b95780639b81788b146125015780638bceaa181461256457806384f16ca0146125c657806395b66162146126295780639a23842e1461268b5780634fae2f23146126ef578063a9f2812f14612752578063f93a1716146127b6576199995f5260205ffd5b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa1561253e57905080602001519051612543565b50505f5f5b9082026127100291900390910290046001015f516020019081525f52612ac1565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa156125a1579050806020015190516125a6565b50505f5f5b82026127100291900390910290046001015f516020019081525f52612ac1565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa1561260357905080602001519051612608565b50505f5f5b9090919092029182029190612710020190045f516020019081525f52612ac1565b5083600c016126f2908060240151906004015161010051630902f1ac8152606081600483601c01855afa156126665790508060200151905161266b565b50505f5f5b90919092029182029190612710020190045f516020019081525f52612ac1565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa156126c9579050806020015190516126ce565b50505f5f5b9082026127100291900390910290046001015f516020019081525f52612ac1565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa1561272d57905080602001519051612732565b50505f5f5b82026127100291900390910290046001015f516020019081525f52612ac1565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa1561279057905080602001519051612795565b50505f5f5b9090919092029182029190612710020190045f516020019081525f52612ac1565b5083600c0180602401518160440151916004015161010051630902f1ac8152606081600483601c01855afa156127f4579050806020015190516127f9565b50505f5f5b90919092029182029190612710020190045f516020019081525f52612ac1565b5083600c0180600401518160240151826044015182610100516370a082318152308160200152602081602483601c01855afa15612c1a5751801561285c57600190035b808411156128ba577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b85606401516101005163a9059cbb81529081602001529081604001525f5f604483601c015f865af150505080156128f5575f5f5f5f84415af1505b50505050612ac1565b5083600c0180600401518160240151826044015182610100516370a082318152308160200152602081602483601c01855afa15612c1a5751801561294157600190035b8084111561299f577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b505080156129b1575f5f5f5f84415af1505b505050612ac1565b5083600c0173c02aaa39b223fe8d0a0e5c4f27ead9083c756cc28160040151826024015182610100516370a082318152308160200152602081602483601c01855afa15612c1a57518015612a0c57600190035b80841115612a6a577f424200000000000000000000000000000000000000000000000000000000000060027f08c379a0000000000000000000000000000000000000000000000000000000005f52602060045260245260445260645ffd5b61010051632e1a7d4d81529081602001525f5f602483601c015f73c02aaa39b223fe8d0a0e5c4f27ead9083c756cc25af150508115612aad575f5f5f5f85415af1505b84604401515f5f5f475f945af15050505050505b915050600c01612ad2565b5061ffff015b0181811061167d575b505060205f51f3612c22565b506320c13b0b60e01b5f5260205ff35b50631626ba7e60e01b5f5260205ff35b6020013560601c6024358015612b5e573361010051630902f1ac8152606081600483601c01855afa15612b4257905080602001519051612b47565b50505f5f5b908202612710029190039091029004600101612ba4565b506044353361010051630902f1ac8152606081600483601c01855afa15612b8d57905080602001519051612b92565b50505f5f5b82026127100291900390910290046001015b336101005163a9059cbb81529081602001529081604001525f5f604483601c015f865af15050505f5ff35b6020013560601c600435805f9012612be657612beb565b506024355b336101005163a9059cbb81529081602001529081604001525f5f604483601c015f865af15050505f5ff35b5f5ff35b3d5f5f3e3d5ffd5b").unwrap();
    static ref NO_OWNER_DEPLOY_PREFIX : Vec<u8> = hex::decode("612c2380600a3d393df3").unwrap();
}

pub struct MulticallerDeployer {
    code: Bytes,
    owner: Option<Address>,
    address: Option<Address>,
}

impl MulticallerDeployer {
    pub fn with_address(self, address: Address) -> Self {
        Self {
            address: Some(address),
            ..self
        }
    }
    pub fn new() -> Self {
        Self {
            code: Bytes::from(NO_OWNER_CODE.clone()),
            owner: None,
            address: None,
        }
    }

    pub fn account_info(&self) -> AccountState {
        AccountState {
            balance: None,
            code: Some(self.code.clone()),
            nonce: None,
            storage: Default::default(),
        }
    }

    pub async fn deploy<P, T>(self, client: P, priv_key: SecretKey) -> Result<Self>
    where
        T: Transport + Clone,
        P: Provider<T, Ethereum> + Send + Sync + Clone + 'static,
    {
        let block = client.get_block_by_number(BlockNumberOrTag::Latest, false).await.map_err(|e| {
            error!("{e}");
            eyre!("CANNOT_GET_BLOCK")
        })?.ok_or_eyre("NO_BLOCK")?;

        let header = block.header;


        let next_base_fee = BaseFeeParams::ethereum().next_block_base_fee(header.gas_used, header.gas_limit, header.base_fee_per_gas.unwrap_or_default());

        let signer = PrivateKeySigner::from_bytes(&B256::from_slice(priv_key.to_bytes().as_slice()))?;

        let wallet = EthereumWallet::new(signer);

        debug!("{:?} with gas fee {} ", header.number, next_base_fee);

        let signer_address = wallet.default_signer().address();

        let balance = client.get_balance(signer_address).block_id(BlockId::Number(BlockNumberOrTag::Latest)).await.map_err(|e| {
            error!("{e}");
            eyre!("CANNOT_GET_BALANCE")
        })?;

        println!("{} {}", signer_address, balance);
        let nonce = client.get_transaction_count(signer_address).block_id(BlockId::Number(BlockNumberOrTag::Latest)).await.map_err(|e| {
            error!("{e}");
            eyre!("CANNOT_GET_NONCE")
        })?;

        let mut tx_request = TransactionRequest::default()
            .gas_limit(3_000_000)
            .transaction_type(2)
            .max_fee_per_gas(next_base_fee)
            .max_priority_fee_per_gas(1)
            .input(TransactionInput::new(self.code.clone()))
            //.to(Address::ZERO)
            .nonce(nonce);
        tx_request.to = Some(TxKind::Create);


        let tx = tx_request.build(&wallet).await.map_err(|e| {
            error!("{e}");
            eyre!("CANNOT_BUILT_TX")
        })?;

        let pending_tx = client.send_raw_transaction(tx.encoded_2718().as_slice()).await.map_err(|e| {
            error!("{e}");
            eyre!("ERROR_SENDING_TX")
        })?;

        let mut block_number = client.get_block_number().await?;

        let final_block = block_number + 10;
        while block_number < final_block {
            let receipt = client.get_transaction_receipt(*tx.tx_hash()).await?;
            match receipt {
                Some(receipt) => {
                    let address = receipt.contract_address.ok_or_eyre("NOT_DEPLOYED")?;
                    return Ok(Self {
                        address: Some(address),
                        ..self
                    });
                }
                _ => {}
            }
            tokio::time::sleep(Duration::from_secs(12)).await;
            block_number = client.get_block_number().await?;
        }

        Err(eyre!("NO_RECEIPT_FOUND"))
        //let receipt = pending_tx.with_timeout(Some(Duration::new(10, 0))).get_receipt().await.map_err(|_| eyre!("CANNOT_GET_RECEIPT"))?;
        //let receipt = pending_tx.with_timeout(Some(Duration::new(100, 0))).watch().await.map_err(|_| eyre!("CANNOT_GET_RECEIPT"))?;


        //let address = Address::repeat_byte(3);
    }


    pub async fn set_code<P, T>(self, client: P, address: Address) -> Result<Self>
    where
        T: Transport + Clone,
        P: Provider<T, Ethereum> + AnvilProviderExt<T, Ethereum> + Send + Sync + Clone + 'static,
    {
        AnvilProviderExt::set_code(&client, address, self.code.clone()).await.map_err(|_| eyre!("CANNOT_SET_CODE"))?;

        Ok(Self {
            address: Some(address),
            ..self
        })
    }

    pub fn address(&self) -> Option<Address> {
        self.address
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;

    use debug_provider::AnvilControl;

    use super::*;

    #[tokio::test]
    async fn test_deploy() -> Result<()>
    {
        let anvil_provider = Arc::new(AnvilControl::from_node_on_block("ws://falcon.loop:8008/looper".to_string(), 19109956).await?);

        let block = anvil_provider.get_block_by_number(BlockNumberOrTag::Latest, false).await?;
        println!("Block number : {}", block.unwrap().header.number.unwrap_or_default());

        let priv_key = anvil_provider.privkey()?;

        let multicaller = MulticallerDeployer::new();

        let multicaller = multicaller.deploy(anvil_provider.clone(), priv_key).await?;

        println!("{}", multicaller.address.unwrap_or_default());

        Ok(())
    }
}