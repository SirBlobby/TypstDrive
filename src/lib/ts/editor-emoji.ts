import type { PickerCategory } from './editor-symbols';

function category(name: string, entries: [number[], string][]): PickerCategory {
    return {
        name,
        characters: entries.map(([codePoints, emojiName]) => ({ character: String.fromCodePoint(...codePoints), name: emojiName }))
    };
}

export const emojiCategories: PickerCategory[] = [
    category('Smileys', [
        [[0x1f600], 'grinning face'], [[0x1f603], 'smiling face big eyes'], [[0x1f604], 'smiling face smiling eyes'],
        [[0x1f601], 'beaming face'], [[0x1f606], 'laughing'], [[0x1f605], 'sweat smile'], [[0x1f602], 'tears of joy'],
        [[0x1f923], 'rolling on floor laughing'], [[0x1f642], 'slightly smiling'], [[0x1f609], 'winking'], [[0x1f60a], 'blush'],
        [[0x1f607], 'halo innocent'], [[0x1f60d], 'heart eyes'], [[0x1f929], 'star struck'], [[0x1f618], 'kiss'],
        [[0x1f60b], 'yum'], [[0x1f61c], 'winking tongue'], [[0x1f914], 'thinking'], [[0x1f610], 'neutral'],
        [[0x1f611], 'expressionless'], [[0x1f644], 'rolling eyes'], [[0x1f60f], 'smirk'], [[0x1f62c], 'grimacing'],
        [[0x1f60c], 'relieved'], [[0x1f634], 'sleeping'], [[0x1f637], 'mask'], [[0x1f912], 'thermometer sick'],
        [[0x1f92f], 'mind blown exploding head'], [[0x1f973], 'party face'], [[0x1f60e], 'sunglasses cool'],
        [[0x1f913], 'nerd'], [[0x1f615], 'confused'], [[0x1f61f], 'worried'], [[0x1f62e], 'open mouth surprised'],
        [[0x1f633], 'flushed'], [[0x1f97a], 'pleading'], [[0x1f622], 'crying'], [[0x1f62d], 'sobbing'],
        [[0x1f631], 'screaming fear'], [[0x1f624], 'triumph steam'], [[0x1f621], 'pouting angry'], [[0x1f480], 'skull']
    ]),
    category('Gestures', [
        [[0x1f44d], 'thumbs up'], [[0x1f44e], 'thumbs down'], [[0x1f44c], 'ok hand'], [[0x270c, 0xfe0f], 'victory peace'],
        [[0x1f91e], 'crossed fingers'], [[0x1f44b], 'waving hand'], [[0x1f44f], 'clapping'], [[0x1f64c], 'raising hands'],
        [[0x1f64f], 'folded hands please thanks'], [[0x1f4aa], 'flexed biceps strong'], [[0x1f449], 'point right'],
        [[0x1f448], 'point left'], [[0x1f446], 'point up'], [[0x1f447], 'point down'], [[0x270d, 0xfe0f], 'writing hand'],
        [[0x1f440], 'eyes'], [[0x1f9e0], 'brain']
    ]),
    category('Hearts and symbols', [
        [[0x2764, 0xfe0f], 'red heart'], [[0x1f9e1], 'orange heart'], [[0x1f49b], 'yellow heart'], [[0x1f49a], 'green heart'],
        [[0x1f499], 'blue heart'], [[0x1f49c], 'purple heart'], [[0x1f5a4], 'black heart'], [[0x1f494], 'broken heart'],
        [[0x2728], 'sparkles'], [[0x2b50], 'star'], [[0x1f525], 'fire'], [[0x1f4af], 'hundred points'], [[0x2705], 'check mark button'],
        [[0x274c], 'cross mark'], [[0x26a0, 0xfe0f], 'warning'], [[0x2757], 'exclamation'], [[0x2753], 'question'],
        [[0x1f4a1], 'light bulb idea'], [[0x1f4a5], 'collision boom'], [[0x1f389], 'party popper tada'], [[0x1f38a], 'confetti ball'],
        [[0x1f3c6], 'trophy'], [[0x1f947], 'first place medal'], [[0x1f680], 'rocket']
    ]),
    category('Nature', [
        [[0x1f436], 'dog'], [[0x1f431], 'cat'], [[0x1f98a], 'fox'], [[0x1f43b], 'bear'], [[0x1f43c], 'panda'],
        [[0x1f981], 'lion'], [[0x1f438], 'frog'], [[0x1f427], 'penguin'], [[0x1f989], 'owl'], [[0x1f41d], 'bee'],
        [[0x1f98b], 'butterfly'], [[0x1f422], 'turtle'], [[0x1f419], 'octopus'], [[0x1f333], 'tree'], [[0x1f335], 'cactus'],
        [[0x1f340], 'four leaf clover'], [[0x1f338], 'cherry blossom'], [[0x1f33b], 'sunflower'], [[0x1f339], 'rose'],
        [[0x2600, 0xfe0f], 'sun'], [[0x1f319], 'crescent moon'], [[0x26c5], 'sun behind cloud'], [[0x1f327, 0xfe0f], 'rain cloud'],
        [[0x26a1], 'lightning high voltage'], [[0x2744, 0xfe0f], 'snowflake'], [[0x1f308], 'rainbow'], [[0x1f30d], 'globe earth']
    ]),
    category('Food', [
        [[0x1f34e], 'red apple'], [[0x1f34c], 'banana'], [[0x1f347], 'grapes'], [[0x1f353], 'strawberry'], [[0x1f351], 'peach'],
        [[0x1f951], 'avocado'], [[0x1f955], 'carrot'], [[0x1f35e], 'bread'], [[0x1f9c0], 'cheese'], [[0x1f355], 'pizza'],
        [[0x1f354], 'hamburger'], [[0x1f35f], 'fries'], [[0x1f32e], 'taco'], [[0x1f35c], 'ramen noodles'], [[0x1f363], 'sushi'],
        [[0x1f369], 'doughnut'], [[0x1f36a], 'cookie'], [[0x1f382], 'birthday cake'], [[0x1f36b], 'chocolate'],
        [[0x2615], 'coffee hot beverage'], [[0x1f375], 'tea'], [[0x1f37a], 'beer']
    ]),
    category('Objects', [
        [[0x1f4da], 'books'], [[0x1f4d6], 'open book'], [[0x1f4dd], 'memo note'], [[0x270f, 0xfe0f], 'pencil'],
        [[0x1f58a, 0xfe0f], 'pen'], [[0x1f4ce], 'paperclip'], [[0x1f4cc], 'pushpin'], [[0x1f4c5], 'calendar'],
        [[0x1f4c8], 'chart increasing'], [[0x1f4c9], 'chart decreasing'], [[0x1f4ca], 'bar chart'], [[0x1f4bb], 'laptop'],
        [[0x1f5a5, 0xfe0f], 'desktop computer'], [[0x1f4f1], 'mobile phone'], [[0x1f4e7], 'email'], [[0x1f4e6], 'package'],
        [[0x1f511], 'key'], [[0x1f512], 'locked'], [[0x1f513], 'unlocked'], [[0x1f527], 'wrench'], [[0x2699, 0xfe0f], 'gear'],
        [[0x1f52c], 'microscope'], [[0x1f52d], 'telescope'], [[0x1f9ea], 'test tube'], [[0x1f3a8], 'artist palette'],
        [[0x1f3b5], 'musical note'], [[0x23f0], 'alarm clock'], [[0x231b], 'hourglass'], [[0x1f514], 'bell'], [[0x1f393], 'graduation cap']
    ]),
    category('Travel', [
        [[0x1f697], 'car'], [[0x1f68c], 'bus'], [[0x1f686], 'train'], [[0x2708, 0xfe0f], 'airplane'], [[0x1f6b2], 'bicycle'],
        [[0x1f6a2], 'ship'], [[0x1f3e0], 'house'], [[0x1f3eb], 'school'], [[0x1f3e2], 'office building'], [[0x1f3d4, 0xfe0f], 'mountain'],
        [[0x1f3d6, 0xfe0f], 'beach'], [[0x1f5fa, 0xfe0f], 'world map'], [[0x1f9ed], 'compass']
    ])
];
