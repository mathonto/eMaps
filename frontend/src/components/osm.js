import React from "react";
import {Map, Marker, Polyline, TileLayer, Tooltip} from "react-leaflet";
import L from 'leaflet'

export default class Osm extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            coordinates: [48.745168, 9.106684]
        };
    }

    componentDidMount() {
        if (navigator.geolocation) {
            const callback = (position) => {
                const coordinates = [
                    position.coords.latitude,
                    position.coords.longitude
                ];
                this.setState({coordinates});
            };
            navigator.geolocation.getCurrentPosition(callback);
        }
    }

    getMarker() {
        const chargingIcon = L.icon({
            iconUrl: require('../assets/charge.png'),

            iconSize: [40, 40],
            iconAnchor: [30, 30],
            popupAnchor: [-3, -76]
        });
        return chargingIcon;
    }

    getCounter = (el) => {
        let index = 0;
        for (const element of this.props.state.chargingMarkers) {
            index += 1;
            if (element == el) {
                return index;
            }
        }
    }

    render() {
        return (
            <Map center={this.state.coordinates}
                 bounds={this.bounds_exist() && [this.props.state.from.coordinates, this.props.state.to.coordinates]}
                 zoom={16}
                 zoomControl={false}
                 onClick={this.setMarkers}>
                <TileLayer url='https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png'
                           attribution='&amp;copy <a href="http://osm.org/copyright">OpenStreetMap</a> contributors'/>

                {this.props.state.from.coordinates && <Marker position={this.props.state.from.coordinates}/>}
                {this.props.state.to.coordinates && <Marker position={this.props.state.to.coordinates}/>}
                {this.props.state.chargingMarkers.map(el => <Marker position={el} icon={this.getMarker()}>
                    <Tooltip direction='right' offset={[-8, -2]} opacity={1}
                             permanent><span>STOP {this.getCounter(el)}</span></Tooltip></Marker>)}

                {this.props.state.path.length >= 2 && <Polyline positions={this.props.state.path}
                                                                color={'blue'}/>}
            </Map>
        );
    }

    setMarkers = (event) => {
        const coordinates = Object.values(event.latlng);
        const value = {
            name: coordinates,
            coordinates: coordinates
        };

        if (!this.props.state.from.coordinates) {
            this.props.setFrom(value);
        } else if (!this.props.state.to.coordinates) {
            this.props.setTo(value);
        }
    };

    bounds_exist = () => {
        return this.props.state.from.coordinates && this.props.state.to.coordinates;
    };
}
