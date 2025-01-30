import {
    // Status & Indicators
    Star, CheckCircle, XCircle, Clock, Flag, TrendingDown, TrendingUp, Siren, RefreshCcwDot, CircleSlashed, Trash2, TriangleAlert, Gauge,
    // Science, Research & Academia
    Microscope, FlaskRound, Atom, Radiation, Dna, Telescope, Library, GraduationCap, Lightbulb, Glasses, Speech,
    // Technology, Engineering & Mathematics
    Cpu, CircuitBoard, BrainCircuit, Brain, Network, Satellite, Construction, Wrench, Factory, Package, Magnet, Anvil, ShipWheel, Fuel, Rocket, Gem, ChartPie, ChartLine, ChartScatter, Sigma, Triangle, Pyramid, Torus, Cone, CircleDashed, CircleDotDashed,
    // Medicine, Society & Environment
    HeartPulse, Stethoscope, Globe, Landmark, Gavel, Key, Image, Video, Music4
} from 'lucide-svelte/icons';

export const categories = [
    {
        name: "Status",
        icons: {
            key: Key, // Retaining Key here for access/security in a societal context
            star: Star,
            check: CheckCircle,
            'x-circle': XCircle,
            'circle-dashed': CircleDashed,
            'circle-dot-dashed': CircleDotDashed,
            'refresh-ccw-dot': RefreshCcwDot,
            'circle-slashed': CircleSlashed,
            clock: Clock,
            flag: Flag, // Keeping Flag here for status-related uses (e.g., flagging an item)
            'trending-up': TrendingUp,
            'trending-down': TrendingDown,
            siren: Siren,
            'triangle-alert': TriangleAlert,
            construction: Construction,
            gauge: Gauge,
            trash: Trash2,
        }
    },
    {
        name: "Research",
        icons: {
            microscope: Microscope,
            flask: FlaskRound,
            atom: Atom,
            radiation: Radiation,
            dna: Dna,
            telescope: Telescope,
            library: Library,
            graduation: GraduationCap,
            lightbulb: Lightbulb,
            glasses: Glasses,
            speech: Speech,
            brain: Brain,
            'chart-pie': ChartPie,
            'chart-line': ChartLine,
            'chart-scatter': ChartScatter,
            sigma: Sigma,
            pyramid: Pyramid,
            torus: Torus,
            cone: Cone,
        }
    },
    {
        name: "Technology",
        icons: {
            cpu: Cpu,
            circuit: CircuitBoard,
            'brain-circuit': BrainCircuit,
            network: Network,
            satellite: Satellite,
            wrench: Wrench,
            factory: Factory,
            package: Package,
            magnet: Magnet,
            anvil: Anvil,
            'ship-wheel': ShipWheel,
            fuel: Fuel,
            rocket: Rocket,
            gem: Gem,
            'heart-pulse': HeartPulse,
            stethoscope: Stethoscope,
            globe: Globe,
            landmark: Landmark,
            gavel: Gavel,
            image: Image,
            video: Video,
            'music-4': Music4
        }
    },
];

// Create a flat object of all icons for direct access
export const Icons = Object.fromEntries(
    categories.flatMap(category =>
        Object.entries(category.icons)
    )
);

// Create array format for the grid display
export const availableIcons = Object.entries(Icons).map(([name, component]) => ({
    name,
    component
}));